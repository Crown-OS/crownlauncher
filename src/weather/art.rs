//! Glossy weather icons, composed as SVG from a few parts and rasterized once per size.

use super::Condition;

const OPEN: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 64 64">
<defs>
  <radialGradient id="sun" cx="0.38" cy="0.34" r="0.75">
    <stop offset="0" stop-color="#FFF1A6"/><stop offset="0.45" stop-color="#FFC93D"/><stop offset="1" stop-color="#FF9A1F"/>
  </radialGradient>
  <radialGradient id="moon" cx="0.35" cy="0.3" r="0.8">
    <stop offset="0" stop-color="#FFF6CF"/><stop offset="1" stop-color="#F2BE47"/>
  </radialGradient>
  <linearGradient id="cloud" x1="0" y1="16" x2="0" y2="52" gradientUnits="userSpaceOnUse">
    <stop offset="0" stop-color="#FFFFFF"/><stop offset="0.55" stop-color="#F3F7FC"/><stop offset="1" stop-color="#CBD7E6"/>
  </linearGradient>
  <linearGradient id="storm" x1="0" y1="16" x2="0" y2="52" gradientUnits="userSpaceOnUse">
    <stop offset="0" stop-color="#E9EEF5"/><stop offset="1" stop-color="#9DABBF"/>
  </linearGradient>
  <filter id="lift" x="-30%" y="-30%" width="160%" height="170%">
    <feDropShadow dx="0" dy="2.2" stdDeviation="2" flood-color="#4F6787" flood-opacity="0.32"/>
  </filter>
  <filter id="glow" x="-50%" y="-50%" width="200%" height="200%">
    <feDropShadow dx="0" dy="1" stdDeviation="1.6" flood-color="#FF9A1F" flood-opacity="0.45"/>
  </filter>
</defs>
"##;

const SUN: &str = r##"<g stroke="#FFC83A" stroke-width="2.6" stroke-linecap="round">
  <path d="M42 4v4M42 36v4M24 22h4M56 22h4M29.3 9.3l2.8 2.8M51.9 31.9l2.8 2.8M29.3 34.7l2.8-2.8M51.9 12.1l2.8-2.8"/>
</g>
<circle cx="42" cy="22" r="10" fill="url(#sun)" filter="url(#glow)"/>
"##;

const BIG_SUN: &str = r##"<g stroke="#FFC83A" stroke-width="3" stroke-linecap="round">
  <path d="M32 4v6M32 54v6M4 32h6M54 32h6M12.2 12.2l4.2 4.2M47.6 47.6l4.2 4.2M12.2 51.8l4.2-4.2M47.6 16.4l4.2-4.2"/>
</g>
<circle cx="32" cy="32" r="15" fill="url(#sun)" filter="url(#glow)"/>
"##;

const MOON: &str = r##"<path d="M44 8a17 17 0 1 0 12 28A14 14 0 0 1 44 8z" fill="url(#moon)" filter="url(#lift)"/>
<circle cx="18" cy="14" r="1.6" fill="#FFE9A8"/><circle cx="12" cy="26" r="1.1" fill="#FFE9A8"/>
"##;

const SMALL_MOON: &str = r##"<path d="M46 6a11 11 0 1 0 8 18A9 9 0 0 1 46 6z" fill="url(#moon)" filter="url(#lift)"/>
"##;

fn cloud(fill: &str, dx: f32, dy: f32) -> String {
    format!(
        r##"<g transform="translate({dx} {dy})" fill="url(#{fill})" filter="url(#lift)">
  <circle cx="22" cy="37" r="10"/><circle cx="34" cy="31" r="13"/><circle cx="45" cy="39" r="8.5"/>
  <rect x="12" y="37" width="42" height="12" rx="6"/>
</g>
"##
    )
}

const DROPS: &str = r##"<g stroke="#4DA3FF" stroke-width="2.4" stroke-linecap="round" opacity="0.9">
  <path d="M22 53l-2 5M32 53l-2 5M42 53l-2 5"/>
</g>
"##;

const FLAKES: &str = r##"<g fill="#8EC5FF"><circle cx="21" cy="56" r="2.2"/><circle cx="32" cy="58" r="2.2"/><circle cx="43" cy="56" r="2.2"/></g>
"##;

const BOLT: &str = r##"<path d="M34 46l-6 9h5l-3 7 9-11h-5l3-5z" fill="#FFC83A" stroke="#F29A12" stroke-width="0.8" stroke-linejoin="round"/>
"##;

const MIST: &str = r##"<g stroke="#B9C6D6" stroke-width="2.6" stroke-linecap="round"><path d="M14 54h36M20 60h24"/></g>
"##;

/// The icon for `condition`, by night when the sun is down.
pub fn svg(condition: Condition, night: bool) -> String {
    let body = match (condition, night) {
        (Condition::Clear, false) => BIG_SUN.to_owned(),
        (Condition::Clear, true) => MOON.to_owned(),
        (Condition::PartlyCloudy, false) => format!("{SUN}{}", cloud("cloud", -6.0, 4.0)),
        (Condition::PartlyCloudy, true) => format!("{SMALL_MOON}{}", cloud("cloud", -6.0, 4.0)),
        (Condition::Cloudy, _) => cloud("cloud", 0.0, 0.0),
        (Condition::Rain, _) => format!("{}{DROPS}", cloud("cloud", 0.0, -4.0)),
        (Condition::Snow, _) => format!("{}{FLAKES}", cloud("cloud", 0.0, -4.0)),
        (Condition::Thunder, _) => format!("{}{BOLT}", cloud("storm", 0.0, -6.0)),
        (Condition::Fog, _) => format!("{}{MIST}", cloud("cloud", 0.0, -6.0)),
    };
    format!("{OPEN}{body}</svg>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::icon::rasterize_svg;

    #[test]
    fn every_icon_rasterizes_with_ink() {
        let conditions = [
            Condition::Clear,
            Condition::PartlyCloudy,
            Condition::Cloudy,
            Condition::Rain,
            Condition::Thunder,
            Condition::Snow,
            Condition::Fog,
        ];
        for condition in conditions {
            for night in [false, true] {
                let pixels = match rasterize_svg(svg(condition, night).as_bytes(), 32, 32) {
                    Ok(pixels) => pixels,
                    Err(error) => panic!("{condition:?}: {error:#}"),
                };
                let inked = pixels.rgba.chunks(4).filter(|pixel| pixel[3] > 0).count();
                assert!(
                    inked > 100,
                    "{condition:?} night={night} drew {inked} pixels"
                );
            }
        }
    }
}
