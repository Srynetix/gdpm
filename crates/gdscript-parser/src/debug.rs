use std::str::FromStr;

use tracing_subscriber::{layer::SubscriberExt, EnvFilter};
use tracing_tree::HierarchicalLayer;

use crate::types::{BoxResult, Res, Span};

pub fn init_tracing() -> BoxResult<()> {
    let hierarchical_layer = HierarchicalLayer::new(1)
        .with_targets(true)
        .with_bracketed_fields(true);
    let filter_layer = EnvFilter::from_str("info")?;

    let subscriber = tracing_subscriber::registry()
        .with(hierarchical_layer)
        .with(filter_layer);
    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

pub fn pp_span(i: Span) -> String {
    format!("{}", i.chars().take(30).collect::<String>())
}

pub fn pp_ret<'a, O: std::fmt::Debug>(name: &'static str, result: Res<'a, O>) -> Res<'a, O> {
    match result {
        Ok((rem, o)) => {
            tracing::info!(name = name, o = ?o, rem = pp_span(rem));
            Ok((rem, o))
        }
        Err(e) => {
            tracing::error!(name = name, err = "nope");
            Err(e)
        }
    }
}
