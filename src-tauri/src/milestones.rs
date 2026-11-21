const THRESHOLD_HOURS: [u64; 8] = [1, 5, 10, 25, 50, 100, 250, 500];

pub fn crossed(seconds_before: u64, seconds_after: u64) -> Option<u64> {
    THRESHOLD_HOURS
        .iter()
        .rev()
        .find(|hours| seconds_before < **hours * 3600 && seconds_after >= **hours * 3600)
        .copied()
}

pub fn title(hours: u64) -> String {
    if hours == 1 {
        "Première heure".to_string()
    } else {
        format!("{hours} heures de jeu")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_the_threshold_just_crossed() {
        assert_eq!(crossed(3590, 3602), Some(1));
        assert_eq!(crossed(3602, 3700), None);
        assert_eq!(crossed(35_990, 36_010), Some(10));
        assert_eq!(title(1), "Première heure");
        assert_eq!(title(25), "25 heures de jeu");
    }
}
