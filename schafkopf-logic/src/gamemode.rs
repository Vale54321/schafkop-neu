use crate::deck::{Card, Suit, Rank};

pub enum Gamemode {
    Sauspiel(Suit),
    Solo(Suit),
    Wenz(Suit),
    Geier(Suit),
    Bettel,
    Ramsch
}

impl Gamemode {
    pub fn winning_card<'a>(&self, cards: [&'a Card; 4]) -> &'a Card  {
        match self {
            Gamemode::Sauspiel(_) | Gamemode::Ramsch | Gamemode::Bettel =>
                winner_for_trump(Suit::Herz, cards),
            Gamemode::Solo(solo_suit)   => winner_for_trump(*solo_suit, cards),
            _ => cards[0],
        }
    }
}

fn winner_for_trump<'a>(trump_suit: Suit, cards: [&'a Card; 4]) -> &'a Card {
    if cards.iter().any(|&c| is_trump(c, trump_suit)) {
        // Highest trump wins
        let winner_idx = cards
            .iter()
            .enumerate()
            .filter(|&(_, &c)| is_trump(c, trump_suit))
            .max_by_key(|&(_, &c)| trump_strength(c, trump_suit))
            .map(|(i, _)| i)
            .unwrap_or(0);
        cards[winner_idx]
    } else {
        // No trump: highest of the led suit wins
        let first_suit = cards[0].suit;
        let winner_idx = cards
            .iter()
            .enumerate()
            .filter(|&(_, &c)| c.suit == first_suit)
            .max_by_key(|&(_, &c)| non_trump_strength(c.rank))
            .map(|(i, _)| i)
            .unwrap_or(0);
        cards[winner_idx]
    }
}

// A card is trump in Sauspiel if it's an Ober, an Unter, or of the trump suit.
fn is_trump(card: &Card, trump_suit: Suit) -> bool {
    card.rank == Rank::Ober || card.rank == Rank::Unter || card.suit == trump_suit
}

// Trump strength according to Schafkopf:
// Obers: Eichel > Gras > Herz > Schell
// Unters: Eichel > Gras > Herz > Schell
// Then trump suit cards: A > 10 > K > 9 > 8 > 7
fn trump_strength(card: &Card, trump_suit: Suit) -> u16 {
    match card.rank {
        Rank::Ober => 300 + ober_unter_suit_strength(card.suit),
        Rank::Unter => 200 + ober_unter_suit_strength(card.suit),
        _ if card.suit == trump_suit => 100 + non_trump_strength(card.rank) as u16,
        _ => 0,
    }
}

// Suit precedence for Obers/Unters: Eichel > Gras > Herz > Schell
fn ober_unter_suit_strength(s: Suit) -> u16 {
    match s {
        Suit::Eichel => 4,
        Suit::Gras   => 3,
        Suit::Herz   => 2,
        Suit::Schell => 1,
    }
}

// Non-trump (following suit) strength: A > 10 > K > 9 > 8 > 7
fn non_trump_strength(rank: Rank) -> u8 {
    match rank {
        Rank::Ass    => 6,
        Rank::Zehn   => 5,
        Rank::Koenig => 4,
        Rank::Neun   => 3,
        Rank::Acht   => 2,
        Rank::Sieben => 1,
        _ => 0, // Ober/Unter are trumps, handled elsewhere
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deck::{Card, Suit, Rank};

    fn card(suit: Suit, rank: Rank) -> Card {
        Card { suit, rank }
    }

    #[test]
    fn sauspiel_trump_beats_non_trump() {
        // In Sauspiel, trumps are all Obers, all Unters, and Herz.
        let c0 = card(Suit::Eichel, Rank::Ass);    // lead (non-trump)
        let c1 = card(Suit::Gras,   Rank::Ober);   // trump (Ober)
        let c2 = card(Suit::Herz,   Rank::Zehn);   // trump (Herz suit)
        let c3 = card(Suit::Gras,   Rank::Ass);    // non-trump

        let winner = Gamemode::Sauspiel(Suit::Eichel).winning_card([&c0, &c1, &c2, &c3]);
        // Ober beats Herz trumps due to higher trump tier
        assert_eq!(winner, &c1);
    }

    #[test]
    fn sauspiel_follow_suit_highest_non_trump_when_no_trumps() {
        // No Obers/Unters and no Herz => follow led suit, highest wins (A > 10 > K > 9 > 8 > 7)
        let c0 = card(Suit::Eichel, Rank::Koenig); // lead
        let c1 = card(Suit::Gras,   Rank::Ass);
        let c2 = card(Suit::Eichel, Rank::Ass);    // same suit as lead, highest
        let c3 = card(Suit::Schell, Rank::Zehn);

        let winner = Gamemode::Sauspiel(Suit::Gras).winning_card([&c0, &c1, &c2, &c3]);
        assert_eq!(winner, &c2);
    }

    #[test]
    fn solo_uses_declared_suit_as_trump() {
        // In Solo(Gras), trumps are all Obers, all Unters, and all Gras.
        // Avoid Obers/Unters to focus on the solo suit behaving as trump.
        let c0 = card(Suit::Herz,   Rank::Koenig); // lead
        let c1 = card(Suit::Gras,   Rank::Zehn);   // trump (solo suit)
        let c2 = card(Suit::Eichel, Rank::Ass);
        let c3 = card(Suit::Schell, Rank::Ass);

        let winner = Gamemode::Solo(Suit::Gras).winning_card([&c0, &c1, &c2, &c3]);
        assert_eq!(winner, &c1);
    }
}