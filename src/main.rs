use std::env;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::{Member, Interaction, InteractionResponseType};
use serenity::model::prelude::command::Command;
use serenity::prelude::*;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, StandardFramework};
use songbird::SerenityInit;

pub mod commands_slash;
pub mod voice_commands;
use crate::voice_commands::join::JOIN_COMMAND; 

struct Handler;

#[command]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Esse bot foi criado por um lufano! (lufa-lufa >>> melhor casa de hogwarts :P)").await?;

    Ok(())
}

#[group]
#[commands(about,join)]
struct General;



#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            // Sending a message can fail, due to a network error, an
            // authentication error, or lack of permissions to post in the
            // channel, so log to stdout when some error happens, with a
            // description of it.
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
        }

        if msg.content == "!hi" {
            if let Err(why) = msg.channel_id.say(&ctx.http, 
                format!("Ah, um caloroso 'oi' para você {}, meu caro! Que prazer é receber uma saudação tão amigável. Espero que sua jornada pelo dia seja repleta de surpresas encantadoras e momentos dignos de contos épicos!",
                 msg.author.name)
            ).await {
                println!("Error sending message: {:?}", why);
            }
        }


        if let Err(why) = conexao_com_sqlite::inserir_mensagem(&msg.author.name, &msg.content) {
            println!("Error sending message: {:?}", why);
        }
    }

    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        if let Err(why) = new_member.user.direct_message(&ctx.http, |m| {
            m.content(format!("Seja muito bem-vindo {}, nobre viajante! Sinta-se à vontade para adentrar este recinto e compartilhar suas histórias e experiências. Que nossos caminhos se cruzem de maneira alegre e que nossa amizade cresça como um tesouro raro encontrado em meio a vastas terras desconhecidas!", new_member.user.name))
        }).await {
            println!("Error sending message: {:?}", why);
        };
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command);

            let content = match command.data.name.as_str() {
                "hobbit" => commands_slash::hobbit::run(),
                _ => "not implemented :(".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        
        let guild_command = Command::create_global_application_command(&ctx.http, |command| {
          commands_slash::hobbit::register(command)
        })
        .await;

        println!("I created the following global slash command: {:#?}", guild_command);
    }
}

#[tokio::main]
async fn main() {

    let framework = StandardFramework::new()
    .configure(|c| c.prefix("/"))
    // The `#[group]` (and similarly, `#[command]`) macro generates static instances
    // containing any options you gave it. For instance, the group `name` and its `commands`.
    // Their identifiers, names you can use to refer to these instances in code, are an
    // all-uppercased version of the `name` with a `_GROUP` suffix appended at the end.
    .group(&GENERAL_GROUP);

    // Create the database
    conexao_com_sqlite::criar_banco_de_dados().unwrap();
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::all();

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird() 
        .await
        .expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

pub mod conexao_com_sqlite {
    use rusqlite::{params, Connection, Result};
    use std::fs;

    pub fn criar_banco_de_dados() -> Result<()> {
        let _ = fs::remove_file("banco_de_dados.db");
        let conn = Connection::open("banco_de_dados.db")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS mensagens (
                id INTEGER PRIMARY KEY,
                author TEXT NOT NULL,
                texto TEXT NOT NULL
            )",
            params![],
        )?;

        Ok(())
    }

    pub fn inserir_mensagem(author: &str, texto: &str) -> Result<()> {
        let conn = Connection::open("banco_de_dados.db")?;

        conn.execute(
            "INSERT INTO mensagens (author, texto) VALUES (?1, ?2)",
            params![author, texto],
        )?;

        Ok(())
    }
}
