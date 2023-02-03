use bevy::prelude::*;

pub mod main_menu;
pub mod select_level_menu;

pub const FONT: &str = "Comfortaa-Regular.ttf";

#[allow(clippy::type_complexity)]
pub fn button_interact_visual_system(
    mut button_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, children) in &mut button_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match interaction {
            Interaction::Clicked => {
                text.sections[0].style.color = Color::rgb(0.75, 0.75, 0.75);
            }
            Interaction::Hovered => {
                text.sections[0].style.color = Color::rgb(0.6, 0.6, 0.6);
            }
            Interaction::None => {
                text.sections[0].style.color = Color::BLACK;
            }
        }
    }
}

pub struct MenuPlugin;

#[derive(Resource)]
struct MenuStyles {
    button_style: Style,
    button_text_style: TextStyle,
    title_style: TextStyle,
    layout_node_style: Style,
}

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_styles);
    }
}

fn setup_styles(mut commands: Commands, assets: Res<AssetServer>) {
    let button_style = Style {
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::Px(8.0)),
        margin: UiRect::all(Val::Px(4.0)),
        flex_grow: 1.0,
        ..Default::default()
    };

    let button_text_style = TextStyle {
        font: assets.load(FONT),
        font_size: 24.0,
        color: Color::BLACK,
    };

    let title_style = TextStyle {
        font: assets.load(FONT),
        font_size: 58.0,
        color: Color::BLACK,
    };

    let layout_node_style = Style {
        size: Size::new(Val::Percent(100.0), Val::Auto),
        margin: UiRect::all(Val::Auto),
        align_self: AlignSelf::Center,
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..Default::default()
    };

    commands.insert_resource(MenuStyles {
        button_style,
        button_text_style,
        title_style,
        layout_node_style,
    })
}
