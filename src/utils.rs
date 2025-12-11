use chrono::NaiveDate;
use chrono::Local;

pub fn get_exchange_rate(currency_code: &str) -> f64 {
    match currency_code {
        "PLN" => 1.0,
        "USD" => 4.10,
        "EUR" => 4.35,
        _ => 1.0,
    }
}


pub fn format_currency(amount: f64, currency: &str, _lang: &str) -> String {
    let abs_amount = amount.abs();
    let sign = if amount < 0.0 { "-" } else { "" };
    match currency {
        "USD" => format!("{}${:.2}", sign, abs_amount),
        "EUR" => format!("{}€{:.2}", sign, abs_amount),
        _ => format!("{}{:.2} {}", sign, abs_amount, currency),
    }
}

pub fn format_date_display(date_str: &str, lang: &str) -> String {
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        if lang == "en" {
            return date.format("%m/%d/%Y").to_string();
        } else {
            return date.format("%d.%m.%Y").to_string();
        }
    }
    date_str.to_string()
}

pub fn parsed_date_from_str(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(&format!("{}-01", s), "%Y-%m-%d").unwrap_or_else(|_| Local::now().date_naive())
}

pub fn get_input_style(is_dark: bool) -> &'static str {
    if is_dark {
        "border p-2 rounded w-full bg-gray-700 border-gray-600 text-white placeholder-gray-400"
    } else {
        "border p-2 rounded w-full bg-white border-gray-300 text-black placeholder-gray-500"
    }
}

pub fn get_box_style(is_dark: bool) -> &'static str {
    if is_dark {
        "p-4 rounded-lg border bg-gray-700 border-gray-600"
    } else {
        "p-4 rounded-lg border bg-gray-50 border-gray-200"
    }
}

pub fn get_main_style(dark: bool) -> &'static str {
    if dark { "min-h-screen p-4 md:p-8 font-sans transition-colors duration-300 bg-slate-900 text-slate-100 relative" } 
    else { "min-h-screen p-4 md:p-8 font-sans transition-colors duration-300 bg-slate-50 text-slate-800 relative" }
}

pub fn get_card_style(dark: bool) -> &'static str {
    if dark { "max-w-5xl mx-auto rounded-3xl shadow-2xl overflow-hidden p-6 md:p-10 transition-colors duration-300 bg-slate-800 border border-slate-700" } 
    else { "max-w-5xl mx-auto rounded-3xl shadow-xl overflow-hidden p-6 md:p-10 transition-colors duration-300 bg-white border border-slate-100" }
}

pub fn get_tab_style(active: bool, dark: bool) -> &'static str {
    if active {
        if dark { "flex-1 py-3 px-4 font-bold text-emerald-400 border-b-4 border-emerald-500" } 
        else { "flex-1 py-3 px-4 font-bold text-emerald-600 border-b-4 border-emerald-600" }
    } else {
        "flex-1 py-3 px-4 font-bold text-slate-400 hover:text-slate-500 transition"
    }
}