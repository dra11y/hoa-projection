use std::collections::BTreeMap;

use owo_colors::OwoColorize;
use rust_decimal::{Decimal, RoundingStrategy, dec};
use tabled::{
    Table, Tabled,
    settings::{Alignment, Style},
};

// Westwind's figures:
const BUDGET_2026_WITHOUT_ROUNDING: Decimal = dec!(1_048_374);
// Something is wrong with this figure:
// No way they came up with $2298 increase in assessments when
// the rounded figures result in a decrease.
const BUDGET_2026_WITH_ROUNDING: Decimal = dec!(1_050_672);

const BR1: &str = "1BR";
const BR2: &str = "2BR";
const BR3: &str = "3BR";

#[allow(unused)]
const NUMBER_OF_UNITS: usize = 228;
const NUMBER_OF_YEARS: usize = 10;
const STARTING_VALUES: [(&str, usize, Decimal); 3] = [
    //
    (BR1, NUMBER_OF_UNITS / 3, dec!(321.20)),
    (BR2, NUMBER_OF_UNITS / 3, dec!(400.78)),
    (BR3, NUMBER_OF_UNITS / 3, dec!(480.48)),
];
const INCREASE_PER_YEAR: Decimal = dec!(1.10);
const YEAR_MONTHS: Decimal = dec!(12.);

#[derive(Debug, Clone, Copy)]
struct YearData {
    #[allow(unused)]
    year: usize,
    without_rounding: Decimal,
    with_rounding: Decimal,
    with_rounding_down: Decimal,
}

#[derive(Tabled)]
struct ProjectionRow {
    #[tabled(rename = "Yr")]
    year0: usize,
    #[tabled(rename = "1BR")]
    one_br: String,
    #[tabled(rename = "1BR ROUND")]
    one_br_rnd: String,
    #[tabled(rename = "1BR DOWN")]
    one_br_flr: String,
    #[tabled(rename = "Yr")]
    year1: usize,
    #[tabled(rename = "2BR")]
    two_br: String,
    #[tabled(rename = "2BR ROUND")]
    two_br_rnd: String,
    #[tabled(rename = "2BR DOWN")]
    two_br_flr: String,
    #[tabled(rename = "Yr")]
    year2: usize,
    #[tabled(rename = "3BR")]
    three_br: String,
    #[tabled(rename = "3BR Rnd")]
    three_br_rnd: String,
    #[tabled(rename = "3BR Flr")]
    three_br_flr: String,
}

// Format a Decimal with thousands separators and two decimal places.
// Example: 1234567.8 -> "1,234,567.80"
fn format_amount(value: Decimal) -> String {
    // Keep sign separate so we can insert commas into the magnitude only
    let is_negative = value.is_sign_negative();
    let magnitude = if is_negative { -value } else { value };
    let s = format!("{:.2}", magnitude);
    if let Some((int_part, frac_part)) = s.split_once('.') {
        let mut out = String::new();
        let chars: Vec<char> = int_part.chars().collect();
        for (i, c) in chars.iter().rev().enumerate() {
            if i != 0 && i % 3 == 0 {
                out.push(',');
            }
            out.push(*c);
        }
        let int_with_commas: String = out.chars().rev().collect();
        if is_negative {
            format!("-{}.{}", int_with_commas, frac_part)
        } else {
            format!("{}.{}", int_with_commas, frac_part)
        }
    } else {
        // Fallback (shouldn't happen with formatted string above)
        s
    }
}

fn main() {
    println!("2026: Roxanne's UNIT assessment figures:");
    println!("Table Assumptions:");
    println!("  - without rounding: {BUDGET_2026_WITHOUT_ROUNDING}");
    println!("  -    with rounding: {BUDGET_2026_WITH_ROUNDING}");
    println!(
        "  -       DIFFERENCE:    {}",
        BUDGET_2026_WITH_ROUNDING - BUDGET_2026_WITHOUT_ROUNDING
    );
    println!("Assumption: Same # of each type of unit (because we can't find these numbers).");
    let mut timeseries: BTreeMap<String, Vec<YearData>> = BTreeMap::new();
    let mut total_without_rounding = dec!(0.);
    let mut total_with_rounding = dec!(0.);
    for year in 0..NUMBER_OF_YEARS {
        for (unit_type, count, starting_value) in STARTING_VALUES.iter() {
            if year == 0 {
                let count = Decimal::from(*count);
                let year0_data = YearData {
                    year,
                    without_rounding: starting_value
                        .round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero),
                    with_rounding: starting_value
                        .round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero),
                    with_rounding_down: starting_value.floor(),
                };

                let without_rounding_monthly = count * year0_data.without_rounding;
                let with_rounding_monthly = count * year0_data.with_rounding;

                let without_rounding_yearly = without_rounding_monthly * YEAR_MONTHS;
                let with_rounding_yearly = with_rounding_monthly * YEAR_MONTHS;

                total_without_rounding += without_rounding_yearly;
                total_with_rounding += with_rounding_yearly;

                println!(
                    "without rounding: {unit_type}: {count} * {:.2} = {without_rounding_monthly:.2} * 12 months = {without_rounding_yearly:.2}",
                    year0_data.without_rounding,
                );
                println!(
                    "   with rounding: {unit_type}: {count} * {:.2} = {with_rounding_monthly:.2} * 12 months = {with_rounding_yearly:.2}",
                    year0_data.with_rounding,
                );

                timeseries
                    .entry(unit_type.to_string())
                    .or_default()
                    .push(year0_data);
            }
            let last_values = timeseries
                .get(*unit_type)
                .unwrap()
                .get(year)
                .cloned()
                .unwrap();
            let year_data = YearData {
                year: year + 1,
                without_rounding: (last_values.without_rounding * INCREASE_PER_YEAR)
                    .round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero),
                with_rounding: (last_values.with_rounding * INCREASE_PER_YEAR)
                    .round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero),
                with_rounding_down: (last_values.with_rounding_down * INCREASE_PER_YEAR).floor(),
            };
            timeseries
                .entry(unit_type.to_string())
                .or_default()
                .push(year_data);
        }
    }

    // Format and align totals
    let total_without_str = format_amount(total_without_rounding);
    let total_with_str = format_amount(total_with_rounding);
    let diff_str = format_amount(total_with_rounding - total_without_rounding);
    let width = *[
        total_without_str.len(),
        total_with_str.len(),
        diff_str.len(),
    ]
    .iter()
    .max()
    .unwrap();
    println!(
        "Total without rounding:  {:>width$}",
        total_without_str,
        width = width
    );
    println!(
        "   Total with rounding:  {:>width$}",
        total_with_str,
        width = width
    );
    println!(
        "            DIFFERENCE: {:>width$}",
        diff_str,
        width = width + 1
    );

    println!("===========================");
    println!("Table Assumptions:");
    println!("  - Each unit type increases by 10% each year.");
    println!(r#"  - Rounded values are "carried over" to the next year (worst case scenario)."#);
    println!(r#"  - (in other words, INCREASES GO DOWN THE COLUMNS, NOT ACROSS)"#);

    let mut rows: Vec<ProjectionRow> = Vec::with_capacity(NUMBER_OF_YEARS);
    for year in 1..=NUMBER_OF_YEARS {
        let one = timeseries.get(BR1).and_then(|v| v.get(year - 1)).unwrap();
        let two = timeseries.get(BR2).and_then(|v| v.get(year - 1)).unwrap();
        let three = timeseries.get(BR3).and_then(|v| v.get(year - 1)).unwrap();
        rows.push(ProjectionRow {
            year0: year,
            one_br: format!("{:.2}", one.without_rounding),
            one_br_rnd: format!("{:.2}", one.with_rounding).yellow().to_string(),
            one_br_flr: format!("{:.2}", one.with_rounding_down)
                .purple()
                .to_string(),
            year1: year,
            two_br: format!("{:.2}", two.without_rounding),
            two_br_rnd: format!("{:.2}", two.with_rounding).yellow().to_string(),
            two_br_flr: format!("{:.2}", two.with_rounding_down)
                .purple()
                .to_string(),
            year2: year,
            three_br: format!("{:.2}", three.without_rounding),
            three_br_rnd: format!("{:.2}", three.with_rounding).yellow().to_string(),
            three_br_flr: format!("{:.2}", three.with_rounding_down)
                .purple()
                .to_string(),
        });
    }

    let mut table = Table::new(rows);
    table.with(Style::rounded()).with(Alignment::right());

    println!("{table}");
}
