const FIELD_CODES: [&str; 7] = ["%f", "%F", "%u", "%U", "%i", "%c", "%k"];

pub fn command(exec: &str) -> Option<Vec<String>> {
    let arguments: Vec<String> = split(exec)
        .into_iter()
        .filter(|argument| !FIELD_CODES.contains(&argument.as_str()))
        .map(|argument| argument.replace("%%", "%"))
        .collect();
    (!arguments.is_empty()).then_some(arguments)
}

fn split(exec: &str) -> Vec<String> {
    let mut arguments = Vec::new();
    let mut current = String::new();
    let (mut quoted, mut escaped, mut started) = (false, false, false);
    for character in exec.chars() {
        match character {
            _ if escaped => {
                current.push(character);
                escaped = false;
            }
            '\\' => escaped = true,
            '"' => {
                quoted = !quoted;
                started = true;
            }
            space if space.is_whitespace() && !quoted => {
                if started || !current.is_empty() {
                    arguments.push(std::mem::take(&mut current));
                    started = false;
                }
            }
            other => current.push(other),
        }
    }
    if started || !current.is_empty() {
        arguments.push(current);
    }
    arguments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_codes_are_dropped_and_quotes_kept_together() {
        assert_eq!(
            command(r#"env "MY VAR=1" /opt/app\ dir/app --name=%% %U"#),
            Some(vec![
                "env".to_owned(),
                "MY VAR=1".to_owned(),
                "/opt/app dir/app".to_owned(),
                "--name=%".to_owned(),
            ])
        );
    }

    #[test]
    fn only_field_codes_run_nothing() {
        assert_eq!(command("%F"), None);
    }
}
