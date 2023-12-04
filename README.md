# Stateful flow functions with Dapr

This demo shows how to build a telegram chatbot for GPT4. The application is written in Rust and deployed as a Wasm serverless function on the flows.network platform. It utilizes the following SaaS or PaaS platforms. 


* Telegram API to interact with bots on the telegram platform. 
* OpenAI's Assistant API to interact with GPT4 in threads. 
* The Catalyst Dapr-as-a-Service API to manage application states.


The Catalyst service provides a key-value store itself. But more interestingly, it can use almost any cloud-based storage service as its KV store backend, providing a standard API for multi-cloud applications. 

## Deploy on flows.network

* Fork the GitHub repo for the flow function's source code
* In flows.network
    * Create a flow
    * Import the above forked repo
    * Set variables in the Advanced tab
        * `telegram_token` -- The access token for your telegram bot created by the botfather 
        * `OPENAI_API_KEY` -- The OpenAI API key
        * `openai_assistant_id` -- The ID for the OpenAI assistant you created
        * `catalyst_token` -- The access token for your Catalyst account
        * `catalyst_url` -- The HTTPS access URL for your Catalyst project
        * `catalyst_kvstore` -- The name of your Catalyst application's kvstore
    * Deploy
* Send a message to your telegram bot


