mod deck;
use deck::Deck;

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
}
