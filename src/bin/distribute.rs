use tabled::{
    Table, Tabled,
    settings::{Alignment, Style},
};

#[derive(Tabled)]
struct DistributionRow {
    #[tabled(rename = "Rank")]
    rank: usize,
    #[tabled(rename = "1BR")]
    one_br: i32,
    #[tabled(rename = "2BR")]
    two_br: i32,
    #[tabled(rename = "3BR")]
    three_br: i32,
    #[tabled(rename = "Total Annual")]
    total_annual: String,
    #[tabled(rename = "Difference")]
    difference: String,
    #[tabled(rename = "Error %")]
    error_percent: String,
}

fn format_currency(cents: i32) -> String {
    let is_negative = cents < 0;
    let abs_cents = cents.abs();
    let dollars = abs_cents / 100;
    let remainder = abs_cents % 100;
    let dollar_str = format!("{}", dollars);

    // Add thousands separators manually
    let mut result = String::new();
    let chars: Vec<char> = dollar_str.chars().collect();
    for (i, c) in chars.iter().rev().enumerate() {
        if i != 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }
    let formatted_dollars: String = result.chars().rev().collect();

    if is_negative {
        format!("-${}.{:02}", formatted_dollars, remainder)
    } else {
        format!("${}.{:02}", formatted_dollars, remainder)
    }
}

fn format_dollars(cents: i32) -> String {
    let abs_cents = cents.abs();
    let dollars = abs_cents / 100;
    let dollar_str = format!("{}", dollars);

    // Add thousands separators manually
    let mut result = String::new();
    let chars: Vec<char> = dollar_str.chars().collect();
    for (i, c) in chars.iter().rev().enumerate() {
        if i != 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }
    result.chars().rev().collect()
}

fn main() {
    let min_each_unit = 40;
    let num_results = 10;

    let total_units = 228;
    let total_annual_cents: i32 = 1_048_374_00;
    let annual_1br: i32 = 3_854_40;
    let annual_2br: i32 = 4_809_36;
    let annual_3br: i32 = 5_765_76;

    let mut results: Vec<(i32, i32, i32, i32, i32)> = Vec::new();

    // Collect all valid combinations
    for x in min_each_unit..=(total_units - min_each_unit) {
        for y in min_each_unit..=(total_units - x - min_each_unit) {
            let z = total_units - x - y;
            if z >= min_each_unit {
                let total_fees = annual_1br * x + annual_2br * y + annual_3br * z;
                // let total_fees = (total_fees / 100) * 100;
                if total_fees < total_annual_cents {
                    continue;
                }
                let difference = total_fees - total_annual_cents;
                results.push((x, y, z, total_fees, difference));
            }
        }
    }

    // Sort by absolute difference (closest to target first)
    results.sort_by_key(|(_, _, _, _, diff)| diff.abs());

    // Take the top N results
    let top_results = results.into_iter().take(num_results).collect::<Vec<_>>();

    if top_results.is_empty() {
        eprintln!("No solutions found.");
        std::process::exit(1);
    }

    println!(
        "Top {} unit distributions closest to target (${}):",
        num_results,
        format_dollars(total_annual_cents)
    );
    println!();

    // Create table rows
    let mut table_rows: Vec<DistributionRow> = Vec::new();
    for (rank, (x, y, z, total_fees, difference)) in top_results.iter().enumerate() {
        let error_percent = (*difference as f64 / total_annual_cents as f64) * 100.0;
        table_rows.push(DistributionRow {
            rank: rank + 1,
            one_br: *x,
            two_br: *y,
            three_br: *z,
            total_annual: format_currency(*total_fees),
            difference: format_currency(*difference),
            error_percent: format!("{:.3}%", error_percent),
        });
    }

    let mut table = Table::new(table_rows);
    table.with(Style::rounded()).with(Alignment::center());

    println!("{table}");
}
