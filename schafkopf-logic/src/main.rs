use bevy::{
    asset::{AssetMetaCheck, AssetPlugin, RenderAssetUsages},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy::ecs::relationship::Relationship;
use schafkopf_logic::{
    deck::{Card, Deck, Rank, Suit},
    gamemode::Gamemode,
    player::{HumanPlayer, InternalPlayer},
};



const CARD_TEXTURE_WIDTH: usize = 96;
const CARD_TEXTURE_HEIGHT: usize = 135;
const CARD_WORLD_SIZE: Vec2 = Vec2::new(96.0, 135.0);
const ICON_OFFSET_TL: Vec2 = Vec2::new(-CARD_WORLD_SIZE.x * 0.5 + 16.0,  CARD_WORLD_SIZE.y * 0.5 - 20.0);
const ICON_OFFSET_BR: Vec2 = Vec2::new( CARD_WORLD_SIZE.x * 0.5 - 16.0, -CARD_WORLD_SIZE.y * 0.5 + 20.0);
const GLYPH_WIDTH: usize = 5;
const GLYPH_HEIGHT: usize = 7;
const GLYPH_STRIDE: usize = 6;
const SUIT_ICON_PX: usize = 32;
const LABEL_MARGIN_X: usize = 14;
const LABEL_MARGIN_Y: usize = 8;
const LABEL_TEXT_GAP: usize = 4;

#[derive(Resource)]
struct CurrentGamemode(Gamemode);

// Resource to hold the currently clicked card label
#[derive(Resource, Default)]
struct ClickedLabel(pub Option<String>);

// Marker for the UI text that shows the clicked card name
#[derive(Component)]
struct ClickText;

#[derive(Resource)]
struct SuitAtlas {
    texture: Handle<Image>,
    layout:  Handle<TextureAtlasLayout>,
}

impl SuitAtlas {
    fn load(
        asset_server: &AssetServer,
        layouts: &mut Assets<TextureAtlasLayout>,
    ) -> Self {
        let texture: Handle<Image> = asset_server.load("symbole.png");
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 2, 2, None, None);
        let layout_handle = layouts.add(layout);

        Self { texture, layout: layout_handle }
    }

    fn index_for(&self, suit: Suit) -> usize {
        match suit {
            Suit::Eichel => 0,
            Suit::Gras   => 1,
            Suit::Herz   => 2,
            Suit::Schell => 3,
        }
    }
}

#[derive(Resource)]
struct PlayerHandResource {
    cards: Vec<Card>,
}

#[derive(Component)]
struct PlayerCardVisual {
    card: Card,
    index: usize,
}

// Marker for the base (non-atlas) sprite child under a card parent
#[derive(Component)]
struct BaseCardSprite;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(
                AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }
            ).set(ImagePlugin::default_nearest()))
        .add_systems(Startup, (setup_game, spawn_click_text))
        .add_systems(PostStartup, spawn_player_hand)
        .add_systems(Update, update_click_text)
        .run();
}

fn setup_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2d);

    let atlas = SuitAtlas::load(&asset_server, &mut texture_layouts);
    commands.insert_resource(atlas);

    let mut deck = Deck::new();
    deck.shuffle();
    let [mut hand1, hand2, hand3, hand4] =
        deck.deal_4x8().expect("expected a full deck to deal four hands");

    sort_cards(&mut hand1);

    let mut p1 = HumanPlayer::new(1, "Alice");
    let mut p2 = HumanPlayer::new(2, "Bob");
    let mut p3 = HumanPlayer::new(3, "Clara");
    let mut p4 = HumanPlayer::new(4, "Max");

    p1.set_hand(hand1);
    p2.set_hand(hand2);
    p3.set_hand(hand3);
    p4.set_hand(hand4);

    let mode = Gamemode::Wenz(None);
    commands.insert_resource(CurrentGamemode(mode));

    commands.insert_resource(ClickedLabel::default());

    commands.insert_resource(PlayerHandResource {
        cards: p1.hand().clone(),
    });
}

fn spawn_click_text(mut commands: Commands, _asset_server: Res<AssetServer>) {
    commands.spawn((
        Text::new("click a card"),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextLayout::new_with_justify(Justify::Left),
        Node {
            position_type: PositionType::Absolute,
            top: px(5),
            left: px(5),
            ..default()
        },
        ClickText,
    ));
}

fn spawn_player_hand(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    atlas: Res<SuitAtlas>,
    hand: Res<PlayerHandResource>,
) {
    let spacing = CARD_WORLD_SIZE.x + 5.0;
    let start_x = -(spacing * (hand.cards.len() as f32 - 1.0) / 2.0);
    let y = -200.0;

    for (i, card) in hand.cards.iter().enumerate() {
        let base_handle = create_card_texture(&mut images, card);

        let parent = commands
            .spawn(Transform::from_xyz(start_x + i as f32 * spacing, y, 0.0))
            .observe(on_hover())
            .observe(on_unhover())
            .id();

        commands.entity(parent).with_children(|c| {
            c.spawn((
                Sprite {
                    image: base_handle,
                    custom_size: Some(CARD_WORLD_SIZE),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.0),
                Pickable::default(),
                BaseCardSprite,
            ))
            .observe(on_click_select(*card));

            c.spawn((
                Sprite::from_atlas_image(
                    atlas.texture.clone(),
                    TextureAtlas {
                        layout: atlas.layout.clone(),
                        index: atlas.index_for(card.suit),
                    },
                ),
                Transform::from_xyz(ICON_OFFSET_TL.x, ICON_OFFSET_TL.y, 0.1), // on top
            ));

            c.spawn((
                Sprite::from_atlas_image(
                    atlas.texture.clone(),
                    TextureAtlas {
                        layout: atlas.layout.clone(),
                        index: atlas.index_for(card.suit),
                    },
                ),
                Transform::from_xyz(ICON_OFFSET_BR.x, ICON_OFFSET_BR.y, 0.1),
            ));
        });
    }
}
fn sort_cards(cards: &mut Vec<Card>) {
    cards.sort_by(|a, b| a.suit.cmp(&b.suit).then(a.rank.cmp(&b.rank)));
}


fn create_card_texture(images: &mut Assets<Image>, card: &Card) -> Handle<Image> {
    let mut pixels = vec![0u8; CARD_TEXTURE_WIDTH * CARD_TEXTURE_HEIGHT * 4];

    let background = suit_background(card.suit);
    for chunk in pixels.chunks_exact_mut(4) {
        chunk.copy_from_slice(&background);
    }

    draw_border(&mut pixels, [45, 45, 45, 255]);

    let rank_text = rank_label(card.rank);

    let ink = [15, 15, 15, 255];
    let rank_text_len = rank_text.chars().count();
    let rank_text_width = if rank_text_len == 0 {
        0
    } else {
        (rank_text_len - 1) * GLYPH_STRIDE + GLYPH_WIDTH
    };

    let top_label_x = LABEL_MARGIN_X;
    let top_label_y = LABEL_MARGIN_Y + SUIT_ICON_PX + LABEL_TEXT_GAP;
    let bottom_label_x = CARD_TEXTURE_WIDTH.saturating_sub(LABEL_MARGIN_X + rank_text_width);
    let bottom_label_y = CARD_TEXTURE_HEIGHT
        .saturating_sub(LABEL_MARGIN_Y + SUIT_ICON_PX + LABEL_TEXT_GAP + GLYPH_HEIGHT);

    draw_text(&mut pixels, top_label_x, top_label_y, rank_text, ink);
    draw_text(&mut pixels, bottom_label_x, bottom_label_y, rank_text, ink);

    let extent = Extent3d {
        width: CARD_TEXTURE_WIDTH as u32,
        height: CARD_TEXTURE_HEIGHT as u32,
        depth_or_array_layers: 1,
    };

    let image = Image::new_fill(
        extent,
        TextureDimension::D2,
        &pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );

    images.add(image)
}

fn draw_border(pixels: &mut [u8], color: [u8; 4]) {
    for x in 0..CARD_TEXTURE_WIDTH {
        set_pixel(pixels, x, 0, color);
        set_pixel(pixels, x, CARD_TEXTURE_HEIGHT - 1, color);
    }

    for y in 0..CARD_TEXTURE_HEIGHT {
        set_pixel(pixels, 0, y, color);
        set_pixel(pixels, CARD_TEXTURE_WIDTH - 1, y, color);
    }
}

fn draw_text(pixels: &mut [u8], start_x: usize, start_y: usize, text: &str, color: [u8; 4]) {
    let mut x = start_x;
    for ch in text.chars() {
        if let Some(bitmap) = glyph_bitmap(ch) {
            draw_glyph(pixels, x, start_y, bitmap, color);
        }
        x += GLYPH_STRIDE;
    }
}

fn draw_glyph(pixels: &mut [u8], start_x: usize, start_y: usize, glyph: [u8; 7], color: [u8; 4]) {
    for (row, pattern) in glyph.iter().enumerate() {
        for col in 0..5 {
            if (pattern >> (4 - col)) & 1 == 1 {
                set_pixel(pixels, start_x + col, start_y + row, color);
            }
        }
    }
}

fn set_pixel(pixels: &mut [u8], x: usize, y: usize, color: [u8; 4]) {
    if x >= CARD_TEXTURE_WIDTH || y >= CARD_TEXTURE_HEIGHT {
        return;
    }
    let index = (y * CARD_TEXTURE_WIDTH + x) * 4;
    pixels[index..index + 4].copy_from_slice(&color);
}

fn glyph_bitmap(ch: char) -> Option<[u8; 7]> {
    match ch {
        '0' => Some([0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110]),
        '1' => Some([0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110]),
        '7' => Some([0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000]),
        '8' => Some([0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110]),
        '9' => Some([0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b11100]),
        'A' => Some([0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001]),
        'K' => Some([0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001]),
        'O' => Some([0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110]),
        'U' => Some([0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110]),
        _ => None,
    }
}

fn rank_label(rank: Rank) -> &'static str {
    match rank {
        Rank::Ass => "A",
        Rank::Zehn => "10",
        Rank::Koenig => "K",
        Rank::Ober => "O",
        Rank::Unter => "U",
        Rank::Neun => "9",
        Rank::Acht => "8",
        Rank::Sieben => "7",
    }
}

fn suit_background(suit: Suit) -> [u8; 4] {
    match suit {
        Suit::Eichel => [245, 235, 220, 255],
        Suit::Gras => [225, 245, 225, 255],
        Suit::Herz => [245, 225, 225, 255],
        Suit::Schell => [245, 240, 210, 255],
    }
}

fn suit_color(suit: Suit) -> [u8; 4] {
    match suit {
        Suit::Eichel => [131, 100, 56, 255],
        Suit::Gras => [62, 120, 54, 255],
        Suit::Herz => [170, 40, 60, 255],
        Suit::Schell => [204, 142, 30, 255],
    }
}

fn on_hover(
) -> impl Fn(
    On<Pointer<Over>>,
    Query<&mut Transform>,
    Query<&Children>,
    Query<(&mut Sprite, Option<&BaseCardSprite>)>,
    Query<&ChildOf>,
)
{
    move |ev, mut q_transform, q_children, mut q_sprite, q_parent| {
        // Determine the card parent entity from the event target
        let mut parent_entity = ev.event_target();
        if let Ok(parent) = q_parent.get(parent_entity) {
            parent_entity = parent.get();
        }

        // Scale the parent
        if let Ok(mut transform) = q_transform.get_mut(parent_entity) {
            transform.scale = Vec3::splat(1.1);
        }

        // Tint only the base sprite child (marked with BaseCardSprite)
        if let Ok(children) = q_children.get(parent_entity) {
            for child in children.iter() {
                if let Ok((mut sprite, maybe_base)) = q_sprite.get_mut(child) {
                    if maybe_base.is_some() {
                        sprite.color = Color::srgb(0.6, 0.6, 0.6);
                    }
                }
            }
        }
    }
}

fn on_unhover(
) -> impl Fn(
    On<Pointer<Out>>,
    Query<&mut Transform>,
    Query<&Children>,
    Query<(&mut Sprite, Option<&BaseCardSprite>)>,
    Query<&ChildOf>,
)
{
    move |ev, mut q_transform, q_children, mut q_sprite, q_parent| {
        // Determine the card parent entity from the event target
        let mut parent_entity = ev.event_target();
        if let Ok(parent) = q_parent.get(parent_entity) {
            parent_entity = parent.get();
        }

        // Reset parent scale
        if let Ok(mut transform) = q_transform.get_mut(parent_entity) {
            transform.scale = Vec3::ONE;
        }

        // Reset tint on the base sprite child
        if let Ok(children) = q_children.get(parent_entity) {
            for child in children.iter() {
                if let Ok((mut sprite, maybe_base)) = q_sprite.get_mut(child) {
                    if maybe_base.is_some() {
                        sprite.color = Color::WHITE;
                    }
                }
            }
        }
    }
}

fn on_click_select(card: Card) -> impl Fn(On<Pointer<Press>>, ResMut<ClickedLabel>) {
    move |_, mut clicked| {
        println!("Clicked on card: {:?}", card);
        clicked.0 = Some(format!("{} {}", card.suit, card.rank));
    }
}

fn update_click_text(mut q: Query<&mut Text, With<ClickText>>, clicked: Res<ClickedLabel>) {
    if let Some(mut text) = q.iter_mut().next() {
        if let Some(label) = &clicked.0 {
            *text = Text::new(label.clone());
        } else {
            *text = Text::new("click a card");
        }
    }
}