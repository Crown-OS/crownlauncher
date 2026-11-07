//! Running an app on the GPU the system does not render with by default — the discrete GPU of a
//! hybrid laptop — as its desktop entry asks with `PrefersNonDefaultGPU=true`. switcheroo-control
//! knows the GPUs and the environment that selects each one.

use std::{collections::HashMap, sync::OnceLock};

use zbus::{
    Connection, Proxy,
    proxy::{Builder, CacheProperties},
    zvariant::{OwnedValue, Value},
};

const SWITCHEROO: &str = "net.hadess.SwitcherooControl";
const SWITCHEROO_PATH: &str = "/net/hadess/SwitcherooControl";
const GPUS: &str = "GPUs";

type GpuDescription = HashMap<String, OwnedValue>;
type Environment = Vec<(String, String)>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuPreference {
    Default,
    NonDefault,
}

impl GpuPreference {
    pub const DESKTOP_ENTRY_KEY: &str = "PrefersNonDefaultGPU";

    /// Reads the value of [`Self::DESKTOP_ENTRY_KEY`].
    pub fn from_desktop_entry(value: Option<&str>) -> Self {
        match value {
            Some("true") => Self::NonDefault,
            _ => Self::Default,
        }
    }

    /// The variables to add to a program's environment to run it on this GPU: none for the
    /// default one, nor when switcheroo-control is absent or knows of a single GPU. Blocks on
    /// the system bus the first time, so call it off the UI thread.
    pub fn environment(self) -> &'static [(String, String)] {
        match self {
            Self::Default => &[],
            Self::NonDefault => non_default_environment(),
        }
    }
}

/// Only an answer is kept, so switcheroo-control installed or started later is still found.
fn non_default_environment() -> &'static [(String, String)] {
    static KNOWN: OnceLock<Environment> = OnceLock::new();
    if let Some(environment) = KNOWN.get() {
        return environment;
    }
    match query_gpus() {
        Ok(gpus) => KNOWN.get_or_init(|| first_non_default_environment(&gpus)),
        Err(error) => {
            log::info!("no GPUs from switcheroo-control: {error}");
            &[]
        }
    }
}

fn query_gpus() -> zbus::Result<Vec<GpuDescription>> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async {
            let system = Connection::system().await?;
            let switcheroo: Proxy<'_> = Builder::new(&system)
                .destination(SWITCHEROO)?
                .path(SWITCHEROO_PATH)?
                .interface(SWITCHEROO)?
                .cache_properties(CacheProperties::No)
                .build()
                .await?;
            switcheroo.get_property(GPUS).await
        })
}

/// switcheroo-control flattens each GPU's `Environment` into `[key, value, key, value, …]`.
fn first_non_default_environment(gpus: &[GpuDescription]) -> Environment {
    let Some(Value::Array(flattened)) = gpus
        .iter()
        .find(|gpu| {
            !matches!(
                gpu.get("Default").map(|value| &**value),
                Some(Value::Bool(true))
            )
        })
        .and_then(|gpu| gpu.get("Environment"))
        .map(|value| &**value)
    else {
        return Vec::new();
    };
    let texts: Vec<&str> = flattened
        .iter()
        .filter_map(|item| match item {
            Value::Str(text) => Some(text.as_str()),
            _ => None,
        })
        .collect();
    texts
        .chunks_exact(2)
        .map(|pair| (pair[0].to_owned(), pair[1].to_owned()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gpu(name: &str, is_default: bool, environment: &[&str]) -> GpuDescription {
        let owned = |value: Value<'_>| match OwnedValue::try_from(value) {
            Ok(owned) => owned,
            Err(error) => panic!("own the test value: {error}"),
        };
        HashMap::from([
            ("Name".to_owned(), owned(Value::from(name))),
            ("Default".to_owned(), owned(Value::from(is_default))),
            (
                "Environment".to_owned(),
                owned(Value::from(environment.to_vec())),
            ),
        ])
    }

    fn pairs(environment: &[(&str, &str)]) -> Environment {
        environment
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect()
    }

    #[test]
    fn the_first_gpu_that_is_not_the_default_is_chosen() {
        let gpus = [
            gpu("Intel", true, &["DRI_PRIME", "pci-0000_00_02_0"]),
            gpu(
                "NVIDIA",
                false,
                &[
                    "__NV_PRIME_RENDER_OFFLOAD",
                    "1",
                    "__GLX_VENDOR_LIBRARY_NAME",
                    "nvidia",
                ],
            ),
            gpu("AMD", false, &["DRI_PRIME", "pci-0000_05_00_0"]),
        ];
        assert_eq!(
            first_non_default_environment(&gpus),
            pairs(&[
                ("__NV_PRIME_RENDER_OFFLOAD", "1"),
                ("__GLX_VENDOR_LIBRARY_NAME", "nvidia"),
            ])
        );
    }

    #[test]
    fn a_single_gpu_changes_nothing() {
        let gpus = [gpu("AMD", true, &["DRI_PRIME", "pci-0000_05_00_0"])];
        assert!(first_non_default_environment(&gpus).is_empty());
        assert!(first_non_default_environment(&[]).is_empty());
    }

    #[test]
    fn a_dangling_key_is_dropped() {
        let gpus = [
            gpu("Intel", true, &[]),
            gpu("AMD", false, &["DRI_PRIME", "1", "ORPHAN"]),
        ];
        assert_eq!(
            first_non_default_environment(&gpus),
            pairs(&[("DRI_PRIME", "1")])
        );
    }

    #[test]
    fn only_true_prefers_the_non_default_gpu() {
        assert_eq!(
            GpuPreference::from_desktop_entry(Some("true")),
            GpuPreference::NonDefault
        );
        assert_eq!(
            GpuPreference::from_desktop_entry(Some("false")),
            GpuPreference::Default
        );
        assert_eq!(
            GpuPreference::from_desktop_entry(None),
            GpuPreference::Default
        );
    }

    #[test]
    fn the_default_gpu_needs_no_lookup() {
        assert!(GpuPreference::Default.environment().is_empty());
    }
}
