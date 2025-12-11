use chrono::{Datelike, Local, NaiveDate};
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use crate::api::invoke;
use crate::models::*;
use crate::translate::t;
use crate::utils::*;

#[component]
pub fn App() -> impl IntoView {
    // 1. STANY APLIKACJI

    // Dane
    let (transactions, set_transactions) = signal::<Vec<Transaction>>(vec![]);
    let (all_limits, set_all_limits) = signal::<HashMap<String, MonthlyLimitData>>(HashMap::new());

    // Konfiguracja
    let (theme, set_theme) = signal("light".to_string());
    let (language, set_language) = signal("pl".to_string());
    let (currency, set_currency) = signal("PLN".to_string());

    // UI State
    let (active_tab, set_active_tab) = signal(0);
    let (show_settings, set_show_settings) = signal(false);
    let (show_yearly, set_show_yearly) = signal(false);
    let (show_save_toast, set_show_save_toast) = signal(false);
    let (is_loaded, set_is_loaded) = signal(false);

    // Formularz
    let (title, set_title) = signal("".to_string());
    let (amount, set_amount) = signal("".to_string());
    let (date, set_date) = signal(Local::now().format("%Y-%m-%d").to_string());
    let (category, set_category) = signal("Ogólne".to_string());

    // Filtry dat
    let (selected_month_str, set_selected_month_str) =
        signal(Local::now().format("%Y-%m").to_string());
    let (limits_month_str, set_limits_month_str) = signal(Local::now().format("%Y-%m").to_string());

    // Listy statyczne
    let categories_list = vec![
        "Codzienne Wydatki",
        "Rachunki",
        "Jedzenie",
        "Auto i Transport",
        "Rozrywka",
        "Nieskategoryzowane",
    ];
    let categories_list_limits = categories_list.clone();

    // Helper: czy ciemny motyw
    let is_dark = move || theme.get() == "dark";

    // 2. STORAGE (Komunikacja z Backendem)

    // Ładowanie
    Effect::new(move |_| {
        spawn_local(async move {
            let result = invoke("load_data", JsValue::NULL).await;
            if let Ok(state) = serde_wasm_bindgen::from_value::<AppState>(result) {
                set_transactions.set(state.transactions);
                set_all_limits.set(state.limits);
                set_theme.set(state.theme);
                set_language.set(state.language);
                set_currency.set(state.currency);
                set_is_loaded.set(true);
            }
        });
    });

    // Zapis
    Effect::new(move |_| {
        let current_transactions = transactions.get();
        let current_limits = all_limits.get();
        let current_theme = theme.get();
        let current_language = language.get();
        let current_currency = currency.get();

        let loaded = is_loaded.get();

        if loaded {
            let state_data = AppState {
                transactions: current_transactions,
                limits: current_limits,
                theme: current_theme,
                language: current_language,
                currency: current_currency,
            };

            #[derive(serde::Serialize)]
            struct SaveArgs {
                state: AppState,
            }

            let args_wrapper = SaveArgs { state: state_data };

            spawn_local(async move {
                let args_js = serde_wasm_bindgen::to_value(&args_wrapper).unwrap();
                let _ = invoke("save_data", args_js).await;
            });
        }
    });

    // 3. OBLICZENIA

    let current_month_limits = Memo::new(move |_| {
        let key = selected_month_str.get();
        all_limits
            .get()
            .get(&key)
            .cloned()
            .unwrap_or(MonthlyLimitData {
                general: 0.0,
                categories: HashMap::new(),
            })
    });

    let editing_month_limits = Memo::new(move |_| {
        let key = limits_month_str.get();
        all_limits
            .get()
            .get(&key)
            .cloned()
            .unwrap_or(MonthlyLimitData {
                general: 0.0,
                categories: HashMap::new(),
            })
    });

    let current_month_total = Memo::new(move |_| {
        let sel_str = selected_month_str.get();
        transactions
            .get()
            .iter()
            .filter(|t| t.date.starts_with(&sel_str))
            .map(|t| t.amount)
            .sum::<f64>()
    });

    let current_month_count = Memo::new(move |_| {
        let sel_str = selected_month_str.get();
        transactions
            .get()
            .iter()
            .filter(|t| t.date.starts_with(&sel_str))
            .count()
    });

    let yearly_summary = Memo::new(move |_| {
        let sel_year = parsed_date_from_str(&selected_month_str.get()).year();
        let mut summary = vec![(0.0, 0.0); 12];
        let txs = transactions.get();
        let limits_map = all_limits.get();

        for t in txs {
            if let Ok(d) = NaiveDate::parse_from_str(&t.date, "%Y-%m-%d") {
                if d.year() == sel_year {
                    summary[(d.month() - 1) as usize].0 += t.amount;
                }
            }
        }
        for month_num in 1..=12 {
            let key = format!("{}-{:02}", sel_year, month_num);
            if let Some(data) = limits_map.get(&key) {
                summary[(month_num - 1) as usize].1 = data.general;
            }
        }
        summary
    });

    // 4. FUNKCJE OBSŁUGUJĄCE ZDARZENIA

    let add_transaction = move |_| {
        let parsed_amount = amount.get().parse::<f64>().unwrap_or(0.0);
        if !title.get().is_empty() && parsed_amount > 0.0 {
            let new_transaction = Transaction {
                id: rand::random(),
                title: title.get(),
                amount: parsed_amount,
                date: date.get(),
                category: category.get(),
            };
            set_transactions.update(|list| list.push(new_transaction));
            set_title.set("".to_string());
            set_amount.set("".to_string());
        }
    };
        let remove_transaction = move |tx: Transaction| {
            let list = transactions.get();
            for t in list {
                if t.id == tx.id {
                    set_transactions.update(|list| {
                        if let Some(pos) = list.iter().position(|x| x.id == t.id) {
                            list.remove(pos);
                        }
                    });
                    break;
                }
            }
    };

    let update_general_limit = move |val_str: String| {
        let val = val_str.parse::<f64>().unwrap_or(0.0).abs();
        let key = limits_month_str.get();
        set_all_limits.update(|map| {
            map.entry(key)
                .or_insert(MonthlyLimitData {
                    general: 0.0,
                    categories: HashMap::new(),
                })
                .general = val;
        });
    };

    let update_cat_limit = move |cat: String, val_str: String| {
        let val = val_str.parse::<f64>().unwrap_or(0.0).abs();
        let key = limits_month_str.get();
        set_all_limits.update(|map| {
            map.entry(key)
                .or_insert(MonthlyLimitData {
                    general: 0.0,
                    categories: HashMap::new(),
                })
                .categories
                .insert(cat, val);
        });
    };

    let change_currency = move |new_currency: String| {
        let old_currency = currency.get();
        if old_currency == new_currency {
            return;
        }
        let ratio = get_exchange_rate(&old_currency) / get_exchange_rate(&new_currency);

        set_transactions.update(|list| {
            for t in list {
                t.amount *= ratio;
            }
        });
        set_all_limits.update(|map| {
            for data in map.values_mut() {
                data.general *= ratio;
                for v in data.categories.values_mut() {
                    *v *= ratio;
                }
            }
        });
        set_currency.set(new_currency);
    };

    let clear_storage = move |_| {
        set_transactions.set(vec![]);
        set_all_limits.set(HashMap::new());
        set_show_settings.set(false);
        spawn_local(async move {
            let _ = invoke(
                "save_data",
                serde_wasm_bindgen::to_value(&AppState::default()).unwrap(),
            )
            .await;
        });
    };

    // 5. WIDOK

    view! {
                    // Fix CSS dla number
                    <style> "input[type=number]::-webkit-inner-spin-button, input[type=number]::-webkit-outer-spin-button { -webkit-appearance: none; margin: 0; }" </style>
            <div
                class:dark=move || is_dark()
                class=move || format!("{} pt-[env(safe-area-inset-top)]", get_main_style(is_dark()))
            >
                <div class=move || get_card_style(is_dark())>

                    // HEADER
                    <div class="flex justify-between items-center mb-6">
                        <div class="w-8"></div>
                        <div class="flex items-center gap-3 border-2 border-slate-200 dark:border-slate-700 rounded-2xl px-6 py-2 bg-white dark:bg-slate-800 transition-colors duration-300">
                            <h1 class="text-3xl font-bold text-emerald-500 drop-shadow-md">
                                "CashFlow"
                            </h1>
                        </div>

                        <button class="text-2xl hover:text-emerald-500 transition opacity-70 hover:opacity-100" on:click=move |_| set_show_settings.set(true)>
                            "\u{2699}\u{FE0F}"
                        </button>
                    </div>

                            // ZAKŁADKI
                            <div class="flex mb-6 padding-b-2 gap-4">
                                <button class={move || get_tab_style(active_tab.get() == 0, is_dark())} on:click=move |_| set_active_tab.set(0)>{move || t("dashboard", &language.get())}</button>
                                <button class={move || get_tab_style(active_tab.get() == 1, is_dark())} on:click=move |_| set_active_tab.set(1)>{move || t("limits", &language.get())}</button>
                            </div>

                            // ZAKŁADKA 1: DASHBOARD
                            <Show when=move || active_tab.get() == 0 fallback=|| view! {}>
                                <div class="flex justify-end mb-4 items-center gap-2">
                                    <label class="text-sm font-bold opacity-70">{move || t("select_month", &language.get())}</label>
                                    <input type="month" class={move || get_input_style(is_dark())} on:input=move |ev| set_selected_month_str.set(event_target_value(&ev)) prop:value=selected_month_str />
                                </div>

                                // Kafelki Statystyk
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-8">
                                    <div class={move || get_box_style(is_dark())}>
                                         <h2 class="text-xs font-bold tracking-wider opacity-60 uppercase mb-1">{move || t("transactions_count", &language.get())}</h2>
                                         <p class="text-3xl font-bold">{current_month_count}</p>
                                    </div>
                                    <div class={move || get_box_style(is_dark())}>
                                         <h2 class="text-xs font-bold tracking-wider opacity-60 uppercase mb-1">{move || t("spent", &language.get())}</h2>
                                         <div class="flex justify-between items-end">
                                            <p class={move || if current_month_limits.get().general > 0.0 && current_month_total.get() > current_month_limits.get().general { "text-3xl font-bold text-red-500" } else { "text-3xl font-bold text-emerald-500" }}>
                                                {move || format_currency(current_month_total.get(), &currency.get(), &language.get())}
                                            </p>
                                            <span class="text-xs opacity-80 mb-1 font-medium border-2 border-black-200 dark:border-white-700 px-2 py-1 rounded">
                                                {move || format!("{}: {}", t("general_limit", &language.get()), format_currency(current_month_limits.get().general, &currency.get(), &language.get()))}
                                            </span>
                                         </div>
                                    </div>
                                </div>

                                // Formularz
                                <div class={move || get_box_style(is_dark())}>
                                    <h3 class="text-xl font-bold mb-4 flex items-center gap-2">
                                        {move || t("add_transaction", &language.get())}
                                    </h3>
                                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                                        <input type="text" placeholder={move || t("name_placeholder", &language.get())} class={move || get_input_style(is_dark())} on:input=move |ev| set_title.set(event_target_value(&ev)) prop:value=title />
                                        
    <input
        type="text"
        inputmode="decimal"
        pattern="[0-9]*[.,]?[0-9]{0,2}"
        placeholder={move || t("amount_placeholder", &language.get())}
        class={move || get_input_style(is_dark())}

        on:keydown=move |ev| {
            let key = ev.key();
            let current_value = amount.get();

            let is_digit = key.chars().all(|c| c.is_ascii_digit());
            let is_decimal = key == "." || key == ",";

            if key.len() > 1 && !is_digit && !is_decimal {
                return;
            }
            if !is_digit && !is_decimal {
                ev.prevent_default();
                return;
            }
            if is_decimal && current_value.contains('.') {
                ev.prevent_default();
                return;
            }
            if is_digit && current_value.contains('.') {
                if let Some(decimal_part) = current_value.split('.').nth(1) {
                    if decimal_part.len() >= 2 {
                        ev.prevent_default();
                    }
                }
            }
        }

        on:input=move |ev| {
            let value = event_target_value(&ev);
            let mut sanitized_value = String::new();
            let mut decimal_point_found = false;

            for c in value.chars() {
                if c.is_ascii_digit() {
                    sanitized_value.push(c);
                } else if (c == '.' || c == ',') && !decimal_point_found {
                    sanitized_value.push('.');
                    decimal_point_found = true;
                }
            }
            if decimal_point_found {
                if let Some((integer_part, decimal_part)) = sanitized_value.split_once('.') {
                    let trimmed_decimal = decimal_part.chars().take(2).collect::<String>();
                    sanitized_value = format!("{}.{}", integer_part, trimmed_decimal);
                }
            }
            set_amount.set(sanitized_value);
        }

        prop:value=amount
    />

                                        <select class={move || get_input_style(is_dark())} on:change=move |ev| set_category.set(event_target_value(&ev)) prop:value=category>
                                            {categories_list.iter().map(|c| {
                                                let c_string = c.to_string();
                                                view! { <option class="text-slate-800 dark:text-slate-800" value=c.to_string()>{move || t(&c_string, &language.get())}</option> }
                                            }).collect::<Vec<_>>()}
                                        </select>

                                        <input type="date" class={move || get_input_style(is_dark())} on:input=move |ev| set_date.set(event_target_value(&ev)) prop:value=date />
                                    </div>
                                    <button class="mt-4 w-full bg-emerald-600 text-white font-bold py-3 px-4 rounded-lg hover:bg-emerald-700 transition shadow-lg shadow-emerald-600/20" on:click=add_transaction>
                                         {move || t("add_transaction", &language.get())}
                                    </button>
                                </div>

                                // Lista
                                <div class="mb-8">
                                    <ul class="divide-y divide-slate-200 dark:divide-slate-700">
                                         <For each=move || { let sel = selected_month_str.get(); transactions.get().into_iter().filter(move |t| t.date.starts_with(&sel)).collect::<Vec<_>>() }
                                        key=move |t| (t.id, currency.get())
                                        
                                        children=move |tx| {
                                            let tx_clone = tx.clone();
                                            view! {
                                            <li class="py-4 flex justify-between items-center px-3 hover:bg-slate-50 
                                            dark:hover:bg-slate-700/50 rounded-lg transition overflow-hidden">
                                            <div class="flex items-center gap-3 flex-1 min-w-0">

                                                <div class="w-2 h-10 bg-emerald-500 rounded-full opacity-50 shrink-0"></div>

                                <div class="min-w-0">
                                    <p class="font-bold text-lg truncate pr-2">{tx.title}</p> <p class="text-sm opacity-60 font-medium flex gap-2 truncate">
                                    <span>{move || format_date_display(&tx.date, &language.get())}</span>
                                    <span class="opacity-50">"•"</span>
                                    <span class="text-emerald-600 dark:text-emerald-400 truncate">{move || t(&tx.category, &language.get())}</span>
                                </p>
                                </div>
                            </div>

                <div class="text-right shrink-0 ml-2">
                    <p class="font-bold text-lg text-red-600 dark:text-red-400 whitespace-nowrap">
                {move || format!("-{}", format_currency(tx.amount, &currency.get(), "any").replace("-", ""))}
            </p>
            
                <button class="mt-4 w-full bg-emerald-600 text-white font-bold py-3 px-4 rounded-lg hover:bg-emerald-700 transition shadow-lg shadow-emerald-600/20" 
                
                on:click=move |_| {
                    remove_transaction(tx_clone.clone());
                 }>
                                         {move || t("remove_transaction", &language.get())}
                                    </button>
                </div>
            </li>
                                        }} />
                                    </ul>
                                </div>

                                // Podsumowanie roczne
                                <div class="text-center mt-8 p-4">
                                     <button class="bg-slate-600 text-white py-2 px-6 rounded-full hover:bg-slate-700 transition text-sm font-bold uppercase tracking-wide" on:click=move |_| set_show_yearly.update(|v| *v = !*v)>
                                        {move || if show_yearly.get() { format!("{} {}", t("hide", &language.get()), t("year_summary", &language.get())) } else { format!("{} {}", t("show", &language.get()), t("year_summary", &language.get())) }}
                                    </button>
                                </div>

                                <Show when=move || show_yearly.get() fallback=|| view! {}>
                                    <div class={move || get_box_style(is_dark())}>
                                        <h3 class="text-lg font-bold mb-4 text-center">{move || t("year_summary", &language.get())} {move || parsed_date_from_str(&selected_month_str.get()).year()}</h3>
                                        <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
                                            {move || {
                                                let summary = yearly_summary.get();
                                                (0..12).map(|i| {
                                                    let (spent, limit) = summary[i];
                                                    let month_key = format!("month_short_{}", i + 1);
                                                    view! {
                                                        <div class={get_box_style(is_dark())}>
                                                            <p class="text-xs opacity-60 uppercase font-bold mb-1">{move || t(&month_key, &language.get())}</p>
                                                            <p class={if limit > 0.0 && spent > limit { "font-bold text-red-500" } else { "font-bold text-emerald-600 dark:text-emerald-400" }}>{format_currency(spent, &currency.get(), &language.get())}</p>
                                                            <p class="text-xs opacity-40">{format!("Limit: {}", format_currency(limit, &currency.get(), &language.get()))}</p>
                                                        </div>
                                                    }
                                                }).collect::<Vec<_>>()
                                            }}
                                        </div>
                                    </div>
                                </Show>
                            </Show>

                            // ZAKŁADKA 2: LIMITY
                            <Show when=move || active_tab.get() == 1 fallback=|| view! {}>
                                <div class={move || get_box_style(is_dark())}>
                                    <div class="flex justify-between items-center mb-6">
                                        <h2 class="text-xl font-bold">{move || t("limits", &language.get())}</h2>
                                        <button class="bg-emerald-600 text-white text-sm font-bold px-6 py-2 rounded-full hover:bg-emerald-700 transition shadow-md" on:click=move |_| {
                                            set_show_save_toast.set(true);
                                            set_timeout(move || set_show_save_toast.set(false), std::time::Duration::from_secs(2));
                                        }>{move || t("save_limits", &language.get())}</button>
                                    </div>
                                    <Show when=move || show_save_toast.get() fallback=|| view! {}>
                                        <div class="mb-6 p-3 bg-emerald-100 border border-emerald-400 text-emerald-800 rounded-lg text-center font-medium shadow-sm">{move || t("saved_msg", &language.get())}</div>
                                    </Show>
                                    <div class="mb-6">
                                        <label class="block font-bold mb-2 text-sm uppercase opacity-70">{move || t("edit_limits_month", &language.get())}</label>
                                        <input type="month" class={move || get_input_style(is_dark())} on:input=move |ev| set_limits_month_str.set(event_target_value(&ev)) prop:value=limits_month_str />
                                    </div>
                                    <hr class="my-6 border-slate-300 dark:border-slate-600"/>
                                    <div class="mb-6">
                                        <span class="font-bold mb-2 text-lg p-2">{move || format!("{} ({})", t("general_limit", &language.get()), currency.get())}</span>
                                        <input type="number" min="0" step="0.01" class={move || get_input_style(is_dark())} on:input=move |ev| update_general_limit(event_target_value(&ev)) prop:value=move || editing_month_limits.get().general/>
                                    </div>
                                    <span class="text-lg font-bold mb-2 p-2">{move || format!("{} ({})", t("cat_limits", &language.get()), currency.get())}</span>
                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                         {categories_list_limits.iter().map(|cat| {
                                            let c_label = cat.to_string();
                                            let c_input = cat.to_string();
                                            let c_val = cat.to_string();
                                            view! {
                                                <div class={get_box_style(is_dark())}>
                                                    <label class="block text-xs font-bold opacity-60 mb-2 uppercase">{move || t(&c_label, &language.get())}</label>
                                                    <input type="number" min="0" step="0.01" class={get_input_style(is_dark())}
                                                        on:input=move |ev| update_cat_limit(c_input.clone(), event_target_value(&ev))
                                                        prop:value=move || editing_month_limits.get().categories.get(&c_val).copied().unwrap_or(0.0) />
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                </div>
                            </Show>

                            // MENU USTAWIEŃ
                            <Show when=move || show_settings.get() fallback=|| view! {}>
                                <div class="fixed inset-0 bg-slate-900/60 backdrop-blur-sm flex justify-center items-center z-50 transition-opacity">
                                    <div class={move || if is_dark() { "p-8 rounded-2xl shadow-2xl w-96 bg-slate-800 text-white border border-slate-700" } else { "p-8 rounded-2xl shadow-2xl w-96 bg-white text-slate-800" }}>
                                        <h2 class="text-2xl font-bold mb-6">{move || t("settings", &language.get())}</h2>

                                        <div class="mb-5">
                                            <label class="block mb-2 font-bold text-sm uppercase opacity-60">{move || t("language", &language.get())}</label>
                                            <select class={move || get_input_style(is_dark())} on:change=move |ev| set_language.set(event_target_value(&ev))>
                                                <option class="text-slate-800 dark:text-white" value="pl" selected={move || language.get() == "pl"}>Polski</option>
                                                <option class="text-slate-800 dark:text-white" value="en" selected={move || language.get() == "en"}>English</option>
                                            </select>
                                        </div>

                                        <div class="mb-5">
                                            <label class="block mb-2 font-bold text-sm uppercase opacity-60">{move || t("currency", &language.get())}</label>
                                            <select class={move || get_input_style(is_dark())} on:change=move |ev| change_currency(event_target_value(&ev))>
                                                <option class="text-slate-800" value="PLN" selected={move || currency.get() == "PLN"}>PLN</option>
                                                <option class="text-slate-800" value="USD" selected={move || currency.get() == "USD"}>USD</option>
                                                <option class="text-slate-800" value="EUR" selected={move || currency.get() == "EUR"}>EUR</option>
                                            </select>
                                        </div>

                                        <div class="mb-8">
                                            <label class="block mb-2 font-bold text-sm uppercase opacity-60">{move || t("theme", &language.get())}</label>
                                            <select class={move || get_input_style(is_dark())} on:change=move |ev| set_theme.set(event_target_value(&ev))>
                                                <option class="text-slate-800" value="light" selected={move || theme.get() == "light"}>{move || t("light", &language.get())}</option>
                                                <option class="text-slate-800" value="dark" selected={move || theme.get() == "dark"}>{move || t("dark", &language.get())}</option>
                                            </select>
                                        </div>

                                        <div class="flex gap-3">
                                            <button class="flex-1 bg-red-500/10 text-red-500 hover:bg-red-500 hover:text-white font-bold py-3 rounded-lg transition" on:click=clear_storage>{move || t("clear_data", &language.get())}</button>
                                            <button class="flex-1 bg-slate-200 text-slate-700 hover:bg-slate-300 dark:bg-slate-700 dark:text-white dark:hover:bg-slate-600 font-bold py-3 rounded-lg transition" on:click=move |_| set_show_settings.set(false)>{move || t("close", &language.get())}</button>
                                        </div>
                                    </div>
                                </div>
                            </Show>
                        </div>
                    </div>
                }
}