mod document;
mod sender;

use document::{Document, Extensions, documents};
use sender::Senders;
use worker::wasm_bindgen::JsCast;
use worker::{Context, Env, ForwardableEmailMessage, HttpMetadata, Result, console_log, event};

#[event(email)]
async fn email(message: ForwardableEmailMessage, env: Env, _ctx: Context) -> Result<()> {
    if !is_allowed(&message, &env)? {
        return reject(&message);
    }
    let documents = documents(&message.raw_bytes().await?, &allowed_extensions(&env)?);
    if documents.is_empty() {
        console_log!("No documents from {}", message.from());
    }
    store_all(&env, documents).await
}

fn is_allowed(message: &ForwardableEmailMessage, env: &Env) -> Result<bool> {
    let senders = Senders::parse(&env.var("ALLOWED_SENDERS")?.to_string());
    Ok(senders.allows(&message.from()))
}

fn reject(message: &ForwardableEmailMessage) -> Result<()> {
    console_log!("Rejected mail from {}", message.from());
    message.set_reject("Unknown sender");
    Ok(())
}

fn allowed_extensions(env: &Env) -> Result<Extensions> {
    Ok(Extensions::parse(
        &env.var("ALLOWED_EXTENSIONS")?.to_string(),
    ))
}

async fn store_all(env: &Env, documents: Vec<Document>) -> Result<()> {
    for document in documents {
        store(env, document).await?;
    }
    Ok(())
}

async fn store(env: &Env, document: Document) -> Result<()> {
    let key = format!("{}-{}", random_uuid()?, document.name);
    env.bucket("INBOX")?
        .put(key.clone(), document.content)
        .http_metadata(HttpMetadata {
            content_type: document.content_type,
            ..HttpMetadata::default()
        })
        .execute()
        .await?;
    console_log!("Stored {key}");
    Ok(())
}

fn random_uuid() -> Result<String> {
    let scope: web_sys::WorkerGlobalScope = worker::js_sys::global().unchecked_into();
    Ok(scope.crypto()?.random_uuid())
}
