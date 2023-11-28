use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::Message,
    prelude::{Context, Mentionable},
    Result, async_trait,
};
use songbird::{EventHandler, EventContext, Event, model::payload::{Speaking, ClientDisconnect}, CoreEvent};

struct Receiver;

impl Receiver {
    pub fn new() -> Self {
        Self{}
    }
}
#[async_trait]
impl EventHandler for Receiver {
    #[allow(unused_variables)]
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        use EventContext as Ctx;
        match ctx {
            Ctx::SpeakingStateUpdate(
                Speaking {speaking, ssrc, user_id, ..}
            ) => {
                // Discord voice calls use RTP, where every sender uses a randomly allocated
                // *Synchronisation Source* (SSRC) to allow receivers to tell which audio
                // stream a received packet belongs to. As this number is not derived from
                // the sender's user_id, only Discord Voice Gateway messages like this one
                // inform us about which random SSRC a user has been allocated. Future voice
                // packets will contain *only* the SSRC.
                //
                // You can implement logic here so that you can differentiate users'
                // SSRCs and map the SSRC to the User ID and maintain this state.
                // Using this map, you can map the `ssrc` in `voice_packet`
                // to the user ID and handle their audio packets separately.
                println!(
                    "Speaking state update: user {:?} has SSRC {:?}, using {:?}",
                    user_id,
                    ssrc,
                    speaking,
                );
            },
            Ctx::SpeakingUpdate(data) => {
                // You can implement logic here which reacts to a user starting
                // or stopping speaking, and to map their SSRC to User ID.
                println!(
                    "Source {} has {} speaking.",
                    data.ssrc,
                    if data.speaking {"started"} else {"stopped"},
                );
            },
            Ctx::VoicePacket(data) => {
                // An event which fires for every received audio packet,
                // containing the decoded data.
                if let Some(audio) = data.audio {
                    println!("Audio packet's first 5 samples: {:?}", audio.get(..5.min(audio.len())));
                    println!(
                        "Audio packet sequence {:05} has {:04} bytes (decompressed from {}), SSRC {}",
                        data.packet.sequence.0,
                        audio.len() * std::mem::size_of::<i16>(),
                        data.packet.payload.len(),
                        data.packet.ssrc,
                    );
                } else {
                    println!("RTP packet, but no audio. Driver may not be configured to decode.");
                }
            },
            Ctx::RtcpPacket(data) => {
                // An event which fires for every received rtcp packet,
                // containing the call statistics and reporting information.
                println!("RTCP packet received: {:?}", data.packet);
            },
            Ctx::ClientDisconnect(
                ClientDisconnect {user_id, ..}
            ) => {
                // You can implement your own logic here to handle a user who has left the
                // voice channel e.g., finalise processing of statistics etc.
                // You will typically need to map the User ID to their SSRC; observed when
                // first speaking.

                println!("Client disconnected: user {:?}", user_id);
            },
            _ => {
                // We won't be registering this struct for any more event classes.
                unimplemented!()
            }
        }

        None
    }
}


#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            check_msg(msg.reply(ctx, "Not in a voice channel").await);

            return Ok(());
        }
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let (handler_lock, conn_result)  = manager.join(guild_id, connect_to).await;

    if let Ok(_) = conn_result {
        // NOTE: this skips listening for the actual connection result.
        let mut handler = handler_lock.lock().await;

        handler.add_global_event(
            CoreEvent::SpeakingStateUpdate.into(),
            Receiver::new(),
        );

        handler.add_global_event(
            CoreEvent::SpeakingUpdate.into(),
            Receiver::new(),
        );

        handler.add_global_event(
            CoreEvent::VoicePacket.into(),
            Receiver::new(),
        );

        handler.add_global_event(
            CoreEvent::RtcpPacket.into(),
            Receiver::new(),
        );

        handler.add_global_event(
            CoreEvent::ClientDisconnect.into(),
            Receiver::new(),
        );

        check_msg(msg.channel_id.say(&ctx.http, &format!("Joined {}", connect_to.mention())).await);
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Error joining the channel").await);
    }

    Ok(())
}

fn check_msg(result: Result<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}

