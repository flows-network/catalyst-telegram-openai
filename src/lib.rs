use async_openai::{
    types::{
        CreateMessageRequestArgs, CreateRunRequestArgs, CreateThreadRequestArgs, MessageContent,
        RunStatus,
    },
    Client,
};
use serde_json::json;
use flowsnet_platform_sdk::logger;
use tg_flows::{listen_to_update, update_handler, Telegram, UpdateKind};

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn on_deploy() {
    logger::init();

    let telegram_token = std::env::var("telegram_token").unwrap();
    listen_to_update(telegram_token).await;
}

#[update_handler]
async fn handler(update: tg_flows::Update) {
    logger::init();
    let telegram_token = std::env::var("telegram_token").unwrap();
    let tele = Telegram::new(telegram_token);
    let catalyst_url = std::env::var("catalyst_url").unwrap();
    let catalyst_token = std::env::var("catalyst_token").unwrap();
    let catalyst_kvstore = std::env::var("catalyst_kvstore").unwrap();
    let catalyst_client = dapr::Dapr::new_with_url(catalyst_url, catalyst_token);

    if let UpdateKind::Message(msg) = update.kind {
        let text = msg.text().unwrap_or("");
        let chat_id = msg.chat.id;
        log::info!("chat_id is {}", chat_id);

        if text == "/start" {
            _ = tele.send_message(chat_id, "Hello, I am ready!");
            return;
        }

        let thread_id = match catalyst_client.get_state(&catalyst_kvstore, chat_id.to_string().as_str()).await {
            Ok(ti) => match text == "/restart" {
                true => {
                    delete_thread(ti.as_str().unwrap()).await;
                    let _ = catalyst_client.delete_state(&catalyst_kvstore, chat_id.to_string().as_str()).await;
                    _ = tele.send_message(chat_id, "Great! Lets start a new conversation.");
                    return;
                }
                false => ti.as_str().unwrap().to_owned(),
            },
            Err(_error) => {
                let ti = create_thread().await;
                log::info!("new ti is {}", ti);
                let _ = catalyst_client.save_state(
                    &catalyst_kvstore, 
                    json!([{"key":chat_id.to_string().as_str(), "value":ti.as_str()}])
                ).await;
                ti
            }
        };
        log::info!("thread_id is {}", thread_id);

        let response = run_message(thread_id.as_str(), String::from(text)).await;
        _ = tele.send_message(chat_id, response);
    }
}

async fn create_thread() -> String {
    let client = Client::new();

    let create_thread_request = CreateThreadRequestArgs::default().build().unwrap();

    match client.threads().create(create_thread_request).await {
        Ok(to) => {
            log::info!("New thread (ID: {}) created.", to.id);
            to.id
        }
        Err(e) => {
            panic!("Failed to create thread. {:?}", e);
        }
    }
}

async fn delete_thread(thread_id: &str) {
    let client = Client::new();

    match client.threads().delete(thread_id).await {
        Ok(_) => {
            log::info!("Old thread (ID: {}) deleted.", thread_id);
        }
        Err(e) => {
            log::error!("Failed to delete thread. {:?}", e);
        }
    }
}

async fn run_message(thread_id: &str, text: String) -> String {
    let client = Client::new();
    let assistant_id = std::env::var("openai_assistant_id").unwrap();

    let mut create_message_request = CreateMessageRequestArgs::default().build().unwrap();
    create_message_request.content = text;
    client
        .threads()
        .messages(&thread_id)
        .create(create_message_request)
        .await
        .unwrap();

    let mut create_run_request = CreateRunRequestArgs::default().build().unwrap();
    create_run_request.assistant_id = assistant_id;
    let run_id = client
        .threads()
        .runs(&thread_id)
        .create(create_run_request)
        .await
        .unwrap()
        .id;

    let mut result = Some("Timeout");
    for _ in 0..5 {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let run_object = client
            .threads()
            .runs(&thread_id)
            .retrieve(run_id.as_str())
            .await
            .unwrap();
        result = match run_object.status {
            RunStatus::Queued | RunStatus::InProgress | RunStatus::Cancelling => {
                continue;
            }
            RunStatus::RequiresAction => Some("Action required for OpenAI assistant"),
            RunStatus::Cancelled => Some("Run is cancelled"),
            RunStatus::Failed => Some("Run is failed"),
            RunStatus::Expired => Some("Run is expired"),
            RunStatus::Completed => None,
        };
        break;
    }

    match result {
        Some(r) => String::from(r),
        None => {
            let mut thread_messages = client
                .threads()
                .messages(&thread_id)
                .list(&[("limit", "1")])
                .await
                .unwrap();

            let c = thread_messages.data.pop().unwrap();
            let c = c.content.into_iter().filter_map(|x| match x {
                MessageContent::Text(t) => Some(t.text.value),
                _ => None,
            });

            c.collect()
        }
    }
}
