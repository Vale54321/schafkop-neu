mod deck;
mod gamemode;

use deck::{Deck, Suit, Card, Rank};
use gamemode::Gamemode;

fn main() {
    let mut deck = Deck::new();
    deck.shuffle();

    let [mut hand1, mut hand2, mut hand3, mut hand4] = deck.deal_4x8().unwrap();

    println!("Player 1 has:");
    for card in hand1.iter() {
        println!("{}", card)
    }

    println!("\nPlayer 2 has:");
    for card in hand2.iter() {
        println!("{}", card)
    }

    println!("\nPlayer 3 has:");
    for card in hand3.iter() {
        println!("{}", card)
    }

    println!("\nPlayer 4 has:");
    for card in hand4.iter() {
        println!("{}", card)
    }

    let mode = Gamemode::Solo(Suit::Gras);

    //let trick: [&Card; 4] = [&hand1[0], &hand2[0], &hand3[0], &hand4[0]];
    let c1 = Card { suit: Suit::Schell, rank: Rank::Zehn };
    let c2 = Card { suit: Suit::Schell, rank: Rank::Ass };
    let c3 = Card { suit: Suit::Schell, rank: Rank::Unter };
    let c4 = Card { suit: Suit::Gras, rank: Rank::Sieben };

    let trick: [&Card; 4] = [&c1, &c2, &c3, &c4];


    for card in trick.iter() {
        println!("{}", card)
    }
    println!("\n");

    let winner = mode.winning_card(trick);
    println!("\nWinning card: {}", winner);
}
