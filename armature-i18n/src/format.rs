//! Date and Number Formatting
//!
//! Provides locale-aware formatting for numbers, dates, and currencies.

use crate::Locale;

/// Date formatting style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DateStyle {
    /// Full date (e.g., "Monday, January 1, 2024")
    Full,
    /// Long date (e.g., "January 1, 2024")
    Long,
    /// Medium date (e.g., "Jan 1, 2024")
    #[default]
    Medium,
    /// Short date (e.g., "1/1/24")
    Short,
}

/// Time formatting style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeStyle {
    /// Full time with timezone (e.g., "12:00:00 PM Eastern Standard Time")
    Full,
    /// Long time with timezone abbrev (e.g., "12:00:00 PM EST")
    Long,
    /// Medium time (e.g., "12:00:00 PM")
    #[default]
    Medium,
    /// Short time (e.g., "12:00 PM")
    Short,
}

// ============================================================================
// Number Formatting
// ============================================================================

/// Number formatting configuration.
#[derive(Debug, Clone)]
pub struct NumberFormatter {
    /// Minimum integer digits
    pub min_integer_digits: usize,
    /// Minimum fraction digits
    pub min_fraction_digits: usize,
    /// Maximum fraction digits
    pub max_fraction_digits: usize,
    /// Use grouping separators
    pub use_grouping: bool,
}

impl Default for NumberFormatter {
    fn default() -> Self {
        Self {
            min_integer_digits: 1,
            min_fraction_digits: 0,
            max_fraction_digits: 3,
            use_grouping: true,
        }
    }
}

impl NumberFormatter {
    /// Create a new number formatter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set minimum fraction digits.
    pub fn min_fraction_digits(mut self, digits: usize) -> Self {
        self.min_fraction_digits = digits;
        self
    }

    /// Set maximum fraction digits.
    pub fn max_fraction_digits(mut self, digits: usize) -> Self {
        self.max_fraction_digits = digits;
        self
    }

    /// Set whether to use grouping separators.
    pub fn use_grouping(mut self, use_grouping: bool) -> Self {
        self.use_grouping = use_grouping;
        self
    }

    /// Format a number for the given locale.
    pub fn format(&self, n: f64, locale: &Locale) -> String {
        let (decimal_sep, group_sep) = get_number_separators(locale);
        
        // Format with appropriate decimal places
        let fraction_digits = if n.fract() == 0.0 {
            self.min_fraction_digits
        } else {
            self.max_fraction_digits.max(self.min_fraction_digits)
        };

        let formatted = if fraction_digits > 0 {
            format!("{:.1$}", n, fraction_digits)
        } else {
            format!("{:.0}", n)
        };

        // Split into integer and fraction parts
        let parts: Vec<&str> = formatted.split('.').collect();
        let integer_part = parts[0];
        let fraction_part = parts.get(1).copied();

        // Add grouping separators to integer part
        let grouped_integer = if self.use_grouping {
            add_grouping(integer_part, group_sep)
        } else {
            integer_part.to_string()
        };

        // Combine with locale-appropriate decimal separator
        match fraction_part {
            Some(frac) if !frac.is_empty() && frac.chars().any(|c| c != '0') => {
                format!("{}{}{}", grouped_integer, decimal_sep, frac)
            }
            Some(frac) if self.min_fraction_digits > 0 => {
                format!("{}{}{}", grouped_integer, decimal_sep, frac)
            }
            _ => grouped_integer,
        }
    }
}

/// Format a number for a locale.
///
/// # Example
///
/// ```
/// use armature_i18n::{format_number, Locale};
///
/// assert_eq!(format_number(1234567.89, &Locale::en_us()), "1,234,567.89");
/// assert_eq!(format_number(1234567.89, &Locale::de_de()), "1.234.567,89");
/// assert_eq!(format_number(1234567.89, &Locale::fr_fr()), "1 234 567,89");
/// ```
pub fn format_number(n: f64, locale: &Locale) -> String {
    NumberFormatter::default()
        .max_fraction_digits(2)
        .format(n, locale)
}

/// Format a percentage for a locale.
///
/// # Example
///
/// ```
/// use armature_i18n::{format_percent, Locale};
///
/// assert_eq!(format_percent(0.75, &Locale::en_us()), "75%");
/// assert_eq!(format_percent(0.125, &Locale::de_de()), "12,5%");
/// ```
pub fn format_percent(n: f64, locale: &Locale) -> String {
    let value = n * 100.0;
    let formatted = NumberFormatter::new()
        .max_fraction_digits(1)
        .format(value, locale);
    format!("{}%", formatted)
}

// ============================================================================
// Currency Formatting
// ============================================================================

/// Currency formatting configuration.
#[derive(Debug, Clone)]
pub struct CurrencyFormatter {
    /// Currency code (ISO 4217)
    pub currency_code: String,
    /// Show currency symbol instead of code
    pub use_symbol: bool,
    /// Symbol position (true = before, false = after)
    pub symbol_before: bool,
}

impl CurrencyFormatter {
    /// Create a new currency formatter.
    pub fn new(currency_code: impl Into<String>) -> Self {
        Self {
            currency_code: currency_code.into().to_uppercase(),
            use_symbol: true,
            symbol_before: true,
        }
    }

    /// Set whether to use symbol.
    pub fn use_symbol(mut self, use_symbol: bool) -> Self {
        self.use_symbol = use_symbol;
        self
    }

    /// Format a currency amount.
    pub fn format(&self, amount: f64, locale: &Locale) -> String {
        let (symbol, before) = get_currency_symbol(&self.currency_code, locale);
        
        let formatted = NumberFormatter::new()
            .min_fraction_digits(2)
            .max_fraction_digits(2)
            .format(amount.abs(), locale);

        let sign = if amount < 0.0 { "-" } else { "" };

        if self.use_symbol {
            if before {
                format!("{}{}{}", sign, &symbol, formatted)
            } else {
                format!("{}{} {}", sign, formatted, &symbol)
            }
        } else {
            format!("{}{} {}", sign, formatted, self.currency_code)
        }
    }
}

/// Format a currency amount for a locale.
///
/// # Example
///
/// ```
/// use armature_i18n::{format_currency, Locale};
///
/// assert_eq!(format_currency(99.99, "USD", &Locale::en_us()), "$99.99");
/// assert_eq!(format_currency(99.99, "EUR", &Locale::de_de()), "99,99 €");
/// assert_eq!(format_currency(99.99, "GBP", &Locale::en_gb()), "£99.99");
/// ```
pub fn format_currency(amount: f64, currency_code: &str, locale: &Locale) -> String {
    CurrencyFormatter::new(currency_code).format(amount, locale)
}

// ============================================================================
// Date Formatting
// ============================================================================

/// Date formatting configuration.
#[derive(Debug, Clone, Default)]
pub struct DateFormatter {
    /// Date style
    pub date_style: Option<DateStyle>,
    /// Time style
    pub time_style: Option<TimeStyle>,
}

impl DateFormatter {
    /// Create a new date formatter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set date style.
    pub fn date_style(mut self, style: DateStyle) -> Self {
        self.date_style = Some(style);
        self
    }

    /// Set time style.
    pub fn time_style(mut self, style: TimeStyle) -> Self {
        self.time_style = Some(style);
        self
    }

    /// Format a date (year, month, day).
    pub fn format_date(&self, year: i32, month: u32, day: u32, locale: &Locale) -> String {
        let style = self.date_style.unwrap_or(DateStyle::Medium);
        format_date_impl(year, month, day, style, locale)
    }

    /// Format a time (hour, minute, second).
    pub fn format_time(&self, hour: u32, minute: u32, second: u32, locale: &Locale) -> String {
        let style = self.time_style.unwrap_or(TimeStyle::Medium);
        format_time_impl(hour, minute, second, style, locale)
    }
}

/// Format a date for a locale.
///
/// # Example
///
/// ```
/// use armature_i18n::{format_date, DateStyle, Locale};
///
/// // Default medium style
/// let date = format_date(2024, 1, 15, &Locale::en_us());
/// assert!(date.contains("Jan") && date.contains("15") && date.contains("2024"));
/// ```
pub fn format_date(year: i32, month: u32, day: u32, locale: &Locale) -> String {
    format_date_impl(year, month, day, DateStyle::Medium, locale)
}

fn format_date_impl(year: i32, month: u32, day: u32, style: DateStyle, locale: &Locale) -> String {
    let month_names_short = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", 
                             "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let month_names_long = ["January", "February", "March", "April", "May", "June",
                            "July", "August", "September", "October", "November", "December"];

    let month_idx = (month.saturating_sub(1) as usize).min(11);

    // Determine date order based on locale
    let is_dmy = matches!(locale.language.as_str(), "en" if locale.region.as_deref() == Some("GB"))
        || matches!(locale.language.as_str(), "fr" | "de" | "es" | "it" | "pt" | "ru" | "pl");
    let is_ymd = matches!(locale.language.as_str(), "ja" | "zh" | "ko");

    match style {
        DateStyle::Full => {
            // TODO: Day of week would need calendar calculation
            format!("{} {}, {}", month_names_long[month_idx], day, year)
        }
        DateStyle::Long => {
            format!("{} {}, {}", month_names_long[month_idx], day, year)
        }
        DateStyle::Medium => {
            if is_ymd {
                format!("{}/{}/{}", year, month, day)
            } else if is_dmy {
                format!("{} {} {}", day, month_names_short[month_idx], year)
            } else {
                format!("{} {}, {}", month_names_short[month_idx], day, year)
            }
        }
        DateStyle::Short => {
            if is_ymd {
                format!("{}/{}/{}", year % 100, month, day)
            } else if is_dmy {
                format!("{}/{}/{}", day, month, year % 100)
            } else {
                format!("{}/{}/{}", month, day, year % 100)
            }
        }
    }
}

fn format_time_impl(hour: u32, minute: u32, second: u32, style: TimeStyle, locale: &Locale) -> String {
    // Determine if 12-hour format
    let use_12h = matches!(locale.language.as_str(), "en");

    match style {
        TimeStyle::Full | TimeStyle::Long => {
            if use_12h {
                let (h, period) = if hour == 0 {
                    (12, "AM")
                } else if hour < 12 {
                    (hour, "AM")
                } else if hour == 12 {
                    (12, "PM")
                } else {
                    (hour - 12, "PM")
                };
                format!("{}:{:02}:{:02} {}", h, minute, second, period)
            } else {
                format!("{:02}:{:02}:{:02}", hour, minute, second)
            }
        }
        TimeStyle::Medium => {
            if use_12h {
                let (h, period) = if hour == 0 {
                    (12, "AM")
                } else if hour < 12 {
                    (hour, "AM")
                } else if hour == 12 {
                    (12, "PM")
                } else {
                    (hour - 12, "PM")
                };
                format!("{}:{:02}:{:02} {}", h, minute, second, period)
            } else {
                format!("{:02}:{:02}:{:02}", hour, minute, second)
            }
        }
        TimeStyle::Short => {
            if use_12h {
                let (h, period) = if hour == 0 {
                    (12, "AM")
                } else if hour < 12 {
                    (hour, "AM")
                } else if hour == 12 {
                    (12, "PM")
                } else {
                    (hour - 12, "PM")
                };
                format!("{}:{:02} {}", h, minute, period)
            } else {
                format!("{:02}:{:02}", hour, minute)
            }
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get decimal and grouping separators for a locale.
fn get_number_separators(locale: &Locale) -> (&'static str, &'static str) {
    match locale.language.as_str() {
        // Comma decimal, period grouping
        "de" | "es" | "it" | "pt" | "nl" | "da" | "sv" | "no" | "fi" |
        "pl" | "cs" | "sk" | "hu" | "ro" | "bg" | "el" | "ru" | "uk" |
        "tr" | "id" | "vi" => (",", "."),
        
        // Comma decimal, space grouping (French-speaking)
        "fr" => (",", " "),
        
        // Period decimal, comma grouping (default English-like)
        _ => (".", ","),
    }
}

/// Add grouping separators to an integer string.
fn add_grouping(s: &str, sep: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    
    if len <= 3 {
        return s.to_string();
    }

    let mut result = String::with_capacity(len + (len - 1) / 3);
    
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push_str(sep);
        }
        result.push(*c);
    }

    result
}

/// Get currency symbol and position for a locale.
fn get_currency_symbol(currency_code: &str, locale: &Locale) -> (String, bool) {
    // Symbol before amount (English-style)
    let symbol_before = !matches!(
        locale.language.as_str(),
        "de" | "fr" | "es" | "it" | "pt" | "nl" | "da" | "sv" | "no" | "fi" |
        "pl" | "cs" | "sk" | "hu" | "ro" | "bg" | "el" | "ru" | "uk" | "vi"
    );

    let symbol = match currency_code {
        "USD" => "$",
        "EUR" => "€",
        "GBP" => "£",
        "JPY" => "¥",
        "CNY" => "¥",
        "KRW" => "₩",
        "INR" => "₹",
        "RUB" => "₽",
        "BRL" => "R$",
        "CHF" => "CHF",
        "CAD" => "CA$",
        "AUD" => "A$",
        "HKD" => "HK$",
        "SGD" => "S$",
        "SEK" => "kr",
        "NOK" => "kr",
        "DKK" => "kr",
        "PLN" => "zł",
        "CZK" => "Kč",
        "MXN" => "MX$",
        "THB" => "฿",
        "TWD" => "NT$",
        _ => currency_code,
    };

    (symbol.to_string(), symbol_before)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number_us() {
        let locale = Locale::en_us();
        assert_eq!(format_number(1234567.89, &locale), "1,234,567.89");
        assert_eq!(format_number(1000.0, &locale), "1,000");
    }

    #[test]
    fn test_format_number_german() {
        let locale = Locale::de_de();
        assert_eq!(format_number(1234567.89, &locale), "1.234.567,89");
    }

    #[test]
    fn test_format_number_french() {
        let locale = Locale::fr_fr();
        assert_eq!(format_number(1234567.89, &locale), "1 234 567,89");
    }

    #[test]
    fn test_format_percent() {
        assert_eq!(format_percent(0.75, &Locale::en_us()), "75%");
        assert_eq!(format_percent(0.125, &Locale::de_de()), "12,5%");
    }

    #[test]
    fn test_format_currency() {
        assert_eq!(format_currency(99.99, "USD", &Locale::en_us()), "$99.99");
        assert_eq!(format_currency(99.99, "EUR", &Locale::de_de()), "99,99 €");
        assert_eq!(format_currency(99.99, "GBP", &Locale::en_gb()), "£99.99");
    }

    #[test]
    fn test_format_date() {
        let us = Locale::en_us();
        let date = format_date(2024, 1, 15, &us);
        assert!(date.contains("Jan"));
        assert!(date.contains("15"));
        assert!(date.contains("2024"));
    }

    #[test]
    fn test_format_date_short() {
        let us = Locale::en_us();
        let date = DateFormatter::new()
            .date_style(DateStyle::Short)
            .format_date(2024, 1, 15, &us);
        assert_eq!(date, "1/15/24");

        let gb = Locale::en_gb();
        let date = DateFormatter::new()
            .date_style(DateStyle::Short)
            .format_date(2024, 1, 15, &gb);
        assert_eq!(date, "15/1/24");
    }

    #[test]
    fn test_format_time() {
        let us = Locale::en_us();
        let time = DateFormatter::new()
            .time_style(TimeStyle::Short)
            .format_time(14, 30, 0, &us);
        assert_eq!(time, "2:30 PM");

        let de = Locale::de_de();
        let time = DateFormatter::new()
            .time_style(TimeStyle::Short)
            .format_time(14, 30, 0, &de);
        assert_eq!(time, "14:30");
    }

    #[test]
    fn test_add_grouping() {
        assert_eq!(add_grouping("1234567", ","), "1,234,567");
        assert_eq!(add_grouping("123", ","), "123");
        assert_eq!(add_grouping("1234", " "), "1 234");
    }
}

