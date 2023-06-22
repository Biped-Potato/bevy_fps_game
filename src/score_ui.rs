use bevy::prelude::*;

#[derive(Component)]
pub struct ScoreText {
    pub score: i32,
}

pub fn update_score(mut score_query: Query<(&ScoreText, &mut Text)>) {
    if let Ok((score_text, mut text)) = score_query.get_single_mut() {
        text.sections[0].value = "SCORE: ".to_string() + &score_text.score.to_string();
    }
}
