//! Compiler-version ranges shared by package type routing and conditions.
use super::version_value::Version;

pub(super) fn matches(range: &str, version: &str) -> Result<bool, String> {
    let (installed, precision) = partial(version)?;
    if precision != 3 {
        return Err("Compiler version is not an exact version".into());
    }
    let mut matched = false;
    for branch in range.split("||") {
        if branch.trim().is_empty() {
            continue;
        }
        let tokens: Vec<_> = branch.split_whitespace().collect();
        let mut valid = true;
        if let [left, "-", right] = tokens.as_slice() {
            let (lower, lower_count) = partial(left)?;
            let (upper, count) = partial(right)?;
            valid = (lower_count == 0 || installed >= lower)
                && if count == 0 {
                    true
                } else if count == 3 {
                    installed <= upper
                } else {
                    installed < next(upper, count)?
                };
        } else {
            let mut index = 0;
            while index < tokens.len() {
                let token = tokens[index];
                let combined;
                let token = if matches!(token, ">" | ">=" | "<" | "<=" | "=" | "~" | "^") {
                    index += 1;
                    combined = format!(
                        "{token}{}",
                        tokens
                            .get(index)
                            .ok_or("Missing version comparator operand")?
                    );
                    combined.as_str()
                } else {
                    token
                };
                valid &= comparator(token, &installed)?;
                index += 1;
            }
        }
        matched |= valid;
    }
    Ok(matched || range.split("||").all(|branch| branch.trim().is_empty()))
}

fn comparator(token: &str, installed: &Version) -> Result<bool, String> {
    let (operator, raw) = [">=", "<=", ">", "<", "=", "~", "^"]
        .iter()
        .find_map(|op| token.strip_prefix(op).map(|raw| (*op, raw)))
        .unwrap_or(("", token));
    let (lower, count) = partial(raw)?;
    if count == 0 {
        return Ok(!matches!(operator, ">" | "<"));
    }
    Ok(match operator {
        ">=" => installed >= &if count < 3 { lower.floor() } else { lower },
        "<" => installed < &if count < 3 { lower.floor() } else { lower },
        ">" if count == 3 => installed > &lower,
        ">" => installed >= &next(lower, count)?.floor(),
        "<=" if count == 3 => installed <= &lower,
        "<=" => installed < &next(lower, count)?.floor(),
        "~" => installed >= &lower && installed < &next(lower, count.min(2))?,
        "^" => {
            let upper_count = if lower.core[0] != 0 || count == 1 {
                1
            } else if lower.core[1] != 0 || count == 2 {
                2
            } else {
                3
            };
            installed >= &lower && installed < &next(lower, upper_count)?
        }
        "" | "=" if count == 3 => installed == &lower,
        "" | "=" => installed >= &lower.clone().floor() && installed < &next(lower, count)?.floor(),
        _ => return Err("Invalid version range comparator".into()),
    })
}

fn partial(raw: &str) -> Result<(Version, usize), String> {
    let (raw, build) = raw
        .split_once('+')
        .map_or((raw, None), |(raw, build)| (raw, Some(build)));
    let (raw, prerelease) = raw
        .split_once('-')
        .map_or((raw, None), |(raw, pre)| (raw, Some(pre)));
    let parts: Vec<_> = raw.split('.').collect();
    if (prerelease.is_some() || build.is_some()) && parts.len() != 3 {
        return Err("Version qualifier requires three numeric positions".into());
    }
    if parts.len() > 3 {
        return Err("Invalid compiler version range".into());
    }
    let mut version = [0; 3];
    let mut count = 0;
    let mut wildcard = false;
    for (index, part) in parts.iter().enumerate() {
        if matches!(*part, "*" | "x" | "X") {
            wildcard = true;
            continue;
        }
        if part.is_empty()
            || !part.bytes().all(|byte| byte.is_ascii_digit())
            || (part.len() > 1 && part.starts_with('0'))
        {
            return Err("Unproven compiler version range syntax".into());
        }
        let value = part.parse().map_err(|_| "Version component overflow")?;
        if !wildcard {
            version[index] = value;
            count += 1;
        }
    }
    Ok((Version::new(version, prerelease, build)?, count))
}

fn next(mut version: Version, count: usize) -> Result<Version, String> {
    if count == 0 {
        return Err("Unbounded version endpoint".into());
    }
    version.core[count - 1] = version.core[count - 1]
        .checked_add(1)
        .ok_or("Version endpoint overflow")?;
    version.core[count..].fill(0);
    version.prerelease.clear();
    Ok(version)
}
