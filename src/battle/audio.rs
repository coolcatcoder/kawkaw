use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};

pub fn plugin(app: &mut App) {
    app.add_message::<AudioMessage>()
        .insert_resource(AudioSettings { volume: 0.05 })
        .add_systems(Update, audio_message);
}

#[derive(Resource)]
pub struct AudioSettings {
    volume: f32,
}

#[derive(Message)]
pub enum AudioMessage {
    Sound(&'static str, PlaybackSettings),
}
fn audio_message(
    mut message: MessageReader<AudioMessage>,
    audio_settings: Res<AudioSettings>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for message in message.read() {
        match message {
            AudioMessage::Sound(path, playback_settings) => {
                commands.spawn((
                    AudioPlayer::new(asset_server.load(*path)),
                    PlaybackSettings {
                        mode: PlaybackMode::Despawn,
                        volume: Volume::Linear(
                            playback_settings.volume.to_linear() * audio_settings.volume,
                        ),
                        ..*playback_settings
                    },
                ));
            }
        }
    }
}
