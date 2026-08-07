//! What the model picker knows.
//!
//! Model names move fast, so this file is the single place that needs touching
//! when a provider ships a new generation. Nothing here is load-bearing: the
//! model field is free text end to end, and this catalog only decides what the
//! picker offers before the user overrides it.

use serde::Serialize;

use crate::models::Provider;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    /// The one we'd pick for somebody who doesn't want to think about it.
    Recommended,
    /// Cheaper and quicker, at some cost to the writing.
    Fast,
    /// Slower and pricier, for dense or unusual interfaces.
    Capable,
    /// Still works, still supported, no longer the one to reach for.
    Older,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub note: &'static str,
    pub tier: Tier,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
    pub id: &'static str,
    pub name: &'static str,
    /// One line for the provider card.
    pub blurb: &'static str,
    pub local: bool,
    pub needs_key: bool,
    /// Where a new user goes to get a key.
    pub key_url: &'static str,
    /// What the key tends to look like, so a wrong paste is obvious.
    pub key_hint: &'static str,
    /// Short numbered steps for getting a key in the UI.
    pub key_guide: Vec<&'static str>,
    /// Whether the endpoint field should be shown, and whether it's mandatory.
    pub base_url_editable: bool,
    pub base_url_required: bool,
    pub default_base_url: &'static str,
    pub default_model: &'static str,
    pub models: Vec<ModelInfo>,
}

/// Verified against each provider's model documentation in August 2026.
pub fn models_for(provider: Provider) -> Vec<ModelInfo> {
    match provider {
        Provider::Gemini => vec![
            ModelInfo {
                id: "gemini-3.6-flash",
                name: "Gemini 3.6 Flash",
                note: "Fast, inexpensive and reads interface text well. The right default.",
                tier: Tier::Recommended,
            },
            ModelInfo {
                id: "gemini-3.5-flash-lite",
                name: "Gemini 3.5 Flash Lite",
                note: "Cheapest and quickest. Good for long recordings of simple screens.",
                tier: Tier::Fast,
            },
            ModelInfo {
                id: "gemini-3.1-pro-preview",
                name: "Gemini 3.1 Pro",
                note: "Best at dense dashboards and unfamiliar software. Slower and pricier.",
                tier: Tier::Capable,
            },
        ],
        Provider::OpenAi => vec![
            ModelInfo {
                id: "gpt-5.6-terra",
                name: "GPT-5.6 Terra",
                note: "Balances quality and cost. The right default.",
                tier: Tier::Recommended,
            },
            ModelInfo {
                id: "gpt-5.6-luna",
                name: "GPT-5.6 Luna",
                note: "Cheapest of the 5.6 family, built for high-volume work.",
                tier: Tier::Fast,
            },
            ModelInfo {
                id: "gpt-5.6-sol",
                name: "GPT-5.6 Sol",
                note: "The frontier model. Best on cluttered or ambiguous screens.",
                tier: Tier::Capable,
            },
        ],
        Provider::Anthropic => vec![
            ModelInfo {
                id: "claude-sonnet-5",
                name: "Claude Sonnet 5",
                note: "Strong on high-resolution screenshots. The right default.",
                tier: Tier::Recommended,
            },
            ModelInfo {
                id: "claude-haiku-4-5",
                name: "Claude Haiku 4.5",
                note: "Quicker and cheaper, for straightforward interfaces.",
                tier: Tier::Fast,
            },
            ModelInfo {
                id: "claude-opus-5",
                name: "Claude Opus 5",
                note: "The most capable Claude. Noticeably slower and more expensive.",
                tier: Tier::Capable,
            },
        ],
        Provider::OpenRouter => vec![
            ModelInfo {
                id: "google/gemini-2.5-flash",
                name: "Gemini 2.5 Flash",
                note: "Fast and inexpensive through OpenRouter. A good place to start.",
                tier: Tier::Recommended,
            },
            ModelInfo {
                id: "openai/gpt-4o-mini",
                name: "GPT-4o mini",
                note: "OpenAI's budget vision model. Quick on simple screens.",
                tier: Tier::Fast,
            },
            ModelInfo {
                id: "anthropic/claude-3.5-sonnet",
                name: "Claude 3.5 Sonnet",
                note: "Reliable on dense UI text without paying for the top tier.",
                tier: Tier::Fast,
            },
            ModelInfo {
                id: "openai/gpt-4o",
                name: "GPT-4o",
                note: "OpenAI's flagship vision model. Best when the screen is busy.",
                tier: Tier::Capable,
            },
            ModelInfo {
                id: "anthropic/claude-3.7-sonnet",
                name: "Claude 3.7 Sonnet",
                note: "Strong on fine print, tables and unfamiliar layouts.",
                tier: Tier::Capable,
            },
        ],
        // Ollama's list is whatever you have downloaded, so it comes from the
        // daemon at runtime rather than from here.
        Provider::Ollama => Vec::new(),
        // A custom endpoint serves whatever it serves; the field is free text.
        Provider::Compatible => Vec::new(),
    }
}

pub fn default_model(provider: Provider) -> &'static str {
    match provider {
        Provider::Gemini => "gemini-3.6-flash",
        Provider::OpenAi => "gpt-5.6-terra",
        Provider::Anthropic => "claude-sonnet-5",
        Provider::OpenRouter => "google/gemini-2.5-flash",
        Provider::Ollama => "qwen3-vl:8b",
        Provider::Compatible => "",
    }
}

pub fn provider_info(provider: Provider) -> ProviderInfo {
    let (name, blurb, key_url, key_hint, key_guide) = match provider {
        Provider::Gemini => (
            "Google Gemini",
            "Generous free tier and the quickest way to get going.",
            "https://aistudio.google.com/app/apikey",
            "Starts with “AIza”",
            vec![
                "Open Google AI Studio and sign in with your Google account.",
                "Click Create API key and pick an existing Google Cloud project, or create one.",
                "Copy the key immediately — Google only shows it once.",
                "Paste it below. Steppy stores it in your Mac keychain.",
            ],
        ),
        Provider::OpenAi => (
            "OpenAI",
            "GPT-5.6 and friends, through the standard OpenAI API.",
            "https://platform.openai.com/api-keys",
            "Starts with “sk-”",
            vec![
                "Open the OpenAI Platform and sign in or create an account.",
                "Add a payment method if prompted — even small usage requires billing on file.",
                "Go to API keys and click Create new secret key.",
                "Copy the key and paste it below. Steppy stores it in your Mac keychain.",
            ],
        ),
        Provider::Anthropic => (
            "Anthropic",
            "Claude reads fine print on busy screens particularly well.",
            "https://console.anthropic.com/settings/keys",
            "Starts with “sk-ant-”",
            vec![
                "Open the Anthropic Console and sign in or create an account.",
                "Add billing if you have not used the API before.",
                "Open Settings → API keys and click Create Key.",
                "Copy the key and paste it below. Steppy stores it in your Mac keychain.",
            ],
        ),
        Provider::OpenRouter => (
            "OpenRouter",
            "One key, many models — Gemini, GPT-4o, Claude, and more through a single API.",
            "https://openrouter.ai/keys",
            "Starts with “sk-or-”",
            vec![
                "Open OpenRouter and sign in or create an account.",
                "Add credits if prompted — OpenRouter bills per model you use.",
                "Go to Keys and create an API key.",
                "Copy the key and paste it below. Steppy stores it in your Mac keychain.",
            ],
        ),
        Provider::Ollama => (
            "On this Mac",
            "Runs entirely offline. No key, no cost, nothing leaves the machine.",
            "https://ollama.com/download",
            "",
            Vec::new(),
        ),
        Provider::Compatible => (
            "Custom endpoint",
            "Any server speaking the OpenAI API — LM Studio, vLLM, llama.cpp, or your own gateway.",
            "",
            "Optional, depending on the server",
            vec![
                "Run a server that exposes an OpenAI-compatible chat API on your network or machine.",
                "Enter its base URL below — for example http://localhost:1234/v1.",
                "If the server needs a key, paste it below. Many local servers do not.",
            ],
        ),
    };

    ProviderInfo {
        id: provider.id(),
        name,
        blurb,
        local: provider.is_local(),
        needs_key: provider.needs_key(),
        key_url,
        key_hint,
        key_guide,
        base_url_editable: matches!(provider, Provider::Ollama | Provider::Compatible),
        base_url_required: matches!(provider, Provider::Compatible),
        default_base_url: provider.default_base_url(),
        default_model: default_model(provider),
        models: models_for(provider),
        // `local` is the only field the UI branches on structurally; everything
        // else is copy.
    }
}

pub fn all_providers() -> Vec<ProviderInfo> {
    Provider::ALL.into_iter().map(provider_info).collect()
}

/// Retired model ids, mapped to the closest thing that still exists.
///
/// Somebody who set up Steppy months ago shouldn't open the app one day to a
/// 404 from a model Google turned off; this quietly moves them forward.
pub fn migrate_model(provider: Provider, model: &str) -> Option<&'static str> {
    let replacement = match (provider, model) {
        (Provider::Gemini, "gemini-2.0-flash") => "gemini-3.5-flash-lite",
        (Provider::Gemini, "gemini-2.0-flash-exp") => "gemini-3.5-flash-lite",
        (Provider::Gemini, "gemini-1.5-flash") => "gemini-3.5-flash-lite",
        (Provider::Gemini, "gemini-1.5-pro") => "gemini-3.1-pro-preview",
        (Provider::Gemini, "gemini-2.5-pro") => "gemini-3.1-pro-preview",
        (Provider::Gemini, "gemini-3-pro-preview") => "gemini-3.1-pro-preview",
        (Provider::Gemini, "gemini-3-flash-preview") => "gemini-3.6-flash",
        (Provider::Gemini, "gemini-3.5-flash") => "gemini-3.6-flash",
        (Provider::Gemini, "gemini-2.5-flash") => "gemini-3.5-flash-lite",
        (Provider::OpenAi, "gpt-5.4-mini") => "gpt-5.6-luna",
        _ => return None,
    };
    Some(replacement)
}

/// Vision-capable models worth downloading, roughly ordered by how much machine
/// they need. Sizes are the download size of the default tag.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalCatalogEntry {
    pub id: &'static str,
    pub name: &'static str,
    pub note: &'static str,
    /// Approximate download size in bytes, for the "this will take a while" copy.
    pub size: u64,
    /// Memory the model wants to run comfortably, in bytes.
    pub min_memory: u64,
    pub recommended: bool,
}

const GB: u64 = 1_000_000_000;

/// Download sizes read off the Ollama library in August 2026. They are shown as
/// approximate in the UI, and the real figure replaces them once a pull starts.
pub fn local_catalog() -> Vec<LocalCatalogEntry> {
    vec![
        LocalCatalogEntry {
            id: "moondream",
            name: "Moondream 2B",
            note: "Tiny and quick. Expect terse writing — good for a first pass.",
            size: 1_700_000_000,
            min_memory: 4 * GB,
            recommended: false,
        },
        LocalCatalogEntry {
            id: "qwen3-vl:4b",
            name: "Qwen3-VL 4B",
            note: "Reads on-screen text unusually well for its size.",
            size: 3_300_000_000,
            min_memory: 8 * GB,
            recommended: false,
        },
        LocalCatalogEntry {
            id: "gemma3:4b",
            name: "Gemma 3 4B",
            note: "Small and quick, and good with non-English interfaces.",
            size: 3_300_000_000,
            min_memory: 8 * GB,
            recommended: false,
        },
        LocalCatalogEntry {
            id: "minicpm-v",
            name: "MiniCPM-V 8B",
            note: "Careful with small details. A solid middleweight.",
            size: 5_500_000_000,
            min_memory: 16 * GB,
            recommended: false,
        },
        LocalCatalogEntry {
            id: "qwen3-vl:8b",
            name: "Qwen3-VL 8B",
            note: "The best balance of writing quality and speed for screenshots.",
            size: 6_100_000_000,
            min_memory: 16 * GB,
            recommended: true,
        },
        LocalCatalogEntry {
            id: "gemma4:12b",
            name: "Gemma 4 12B",
            note: "The most natural prose of the local models, if you have the memory.",
            size: 7_600_000_000,
            min_memory: 24 * GB,
            recommended: false,
        },
        LocalCatalogEntry {
            id: "llama3.2-vision:11b",
            name: "Llama 3.2 Vision 11B",
            note: "Strong at pulling dense text out of a busy screen.",
            size: 7_800_000_000,
            min_memory: 24 * GB,
            recommended: false,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_provider_has_a_default_model_it_offers() {
        for provider in Provider::ALL {
            let models = models_for(provider);
            if models.is_empty() {
                continue;
            }
            let default = default_model(provider);
            assert!(
                models.iter().any(|m| m.id == default),
                "{} defaults to `{default}`, which is not in its own list",
                provider.id()
            );
        }
    }

    #[test]
    fn exactly_one_recommended_model_per_provider() {
        for provider in Provider::ALL {
            let models = models_for(provider);
            if models.is_empty() {
                continue;
            }
            let count = models.iter().filter(|m| m.tier == Tier::Recommended).count();
            assert_eq!(count, 1, "{} has {count} recommended models", provider.id());
        }
    }

    #[test]
    fn retired_models_migrate_to_something_we_still_offer() {
        let current = models_for(Provider::Gemini);
        for stale in [
            "gemini-2.0-flash",
            "gemini-1.5-pro",
            "gemini-2.5-pro",
            "gemini-3-pro-preview",
        ] {
            let replacement = migrate_model(Provider::Gemini, stale)
                .unwrap_or_else(|| panic!("`{stale}` has no replacement"));
            assert!(
                current.iter().any(|m| m.id == replacement),
                "`{stale}` migrates to `{replacement}`, which we no longer offer"
            );
        }
    }

    #[test]
    fn models_we_still_offer_are_never_migrated_away() {
        for provider in Provider::ALL {
            for model in models_for(provider) {
                assert!(
                    migrate_model(provider, model.id).is_none(),
                    "`{}` is offered and migrated at the same time",
                    model.id
                );
            }
        }
    }

    #[test]
    fn the_recommended_local_model_is_the_default() {
        let catalog = local_catalog();
        assert_eq!(catalog.iter().filter(|e| e.recommended).count(), 1);
        assert!(catalog
            .iter()
            .any(|e| e.id == default_model(Provider::Ollama) && e.recommended));
    }
}
