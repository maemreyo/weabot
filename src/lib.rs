use anyhow::anyhow;
use serenity::model::application::interaction::Interaction;
use serenity::model::prelude::interaction::InteractionResponseType;
use serenity::{async_trait, model::prelude::GuildId};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_secrets::SecretStore;
use tracing::{error, info};

struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        let guild_id = GuildId(992199576842928138);

        let commands = 
            GuildId::set_application_commands(
                &guild_id,
                &_ctx.http,
                |commands| {
                    commands.create_application_command(|commands| {
                        commands.name("hello").description("Say hello")
                    })
                }
            )
            .await
            .unwrap();
    }

    async fn interaction_create(&self, _ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let response_content = match command.data.name.as_str() {
                "hello" => "hello".to_owned(),
                command => unreachable!("Unknown command: {}", command),
            };

            let create_interaction_response =
                command.create_interaction_response(&_ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.content(response_content)
                        })
                });
            if let Err(why) = create_interaction_response.await {
                eprint!("Cannot respond to slask command: {}", why);
            }
        }
    }
}

#[shuttle_service::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_service::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let token = if let Some(token) = secret_store.get("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let client = Client::builder(&token, intents)
        .event_handler(Bot)
        .await
        .expect("Err creating client");

    Ok(client)
}
