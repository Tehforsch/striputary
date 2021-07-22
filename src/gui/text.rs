use bevy::prelude::*;

use crate::song::Song;

pub fn get_text_bundle_for_song(asset_server: &AssetServer, song: &Song, alignment: HorizontalAlign) -> Text2dBundle {
    Text2dBundle {
        text: Text::with_section(
            &format!("{}\n{}", song.artist, song.title),
            TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 30.0,
                color: Color::BLACK,
            },
            TextAlignment {
                horizontal: alignment,
                ..Default::default()
            },
        ),
        ..Default::default()
    }
}
