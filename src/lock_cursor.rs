use bevy::{prelude::*, window::PrimaryWindow};
#[derive(Resource)]
pub struct CursorLockState {
    pub state: bool,
    pub allow_lock : bool,
}
pub fn lock_cursor_position(
    mut primary_query: Query<&mut Window, With<PrimaryWindow>>,
    mut cursor_lock_state: ResMut<CursorLockState>,
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
) {
    let Ok(mut primary) = primary_query.get_single_mut() else
    {
        return;
    };

    if key.just_pressed(KeyCode::Tab)
    {
        cursor_lock_state.allow_lock = !cursor_lock_state.allow_lock;
    }
    if cursor_lock_state.allow_lock 
    {
        if btn.just_pressed(MouseButton::Left) {
            cursor_lock_state.state = true;
        }
    }
    

    if key.just_pressed(KeyCode::Escape) {
        cursor_lock_state.state = false;
    }

    if cursor_lock_state.state {
        primary.cursor.visible = false;
        let width = primary.width();
        let height = primary.height();
        primary.set_cursor_position(Some(Vec2::new(width / 2., height / 2.)));
    } else {
        primary.cursor.visible = true;
    }
}
