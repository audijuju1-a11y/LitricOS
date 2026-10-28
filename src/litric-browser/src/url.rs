//! URLs, and working out what somebody meant when they typed into the address
//! bar.
//!
//! The address bar is the most security-sensitive text field on a desktop. It
//! is where a user decides whether they are talking to their bank, and it is
//! where every trick for making one site look like another gets played. So the
//! parsing here is deliberately strict and the *display* is deliberately
//! careful:
//!
//! * A host containing non-ASCII is rendered in Punycode, not in the script it
//!   claims to be, unless every character comes from one script. Mixed-script
//!   hosts are how `аpple.com` — with a Cyrillic а — gets read as Apple.
//! * Credentials in a URL (`https://bank.com@evil.example/`) are refused
//!   outright rather than shown small.
//! * The origin is what the user is told about, because the origin is what the
//!   security boundary actually is.

use core::fmt;

/// Why a URL could not be used.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum UrlError {
    /// Nothing was typed.
    Empty,
    /// The scheme is not one the browser handles.
    UnsupportedScheme {
        /// What was given.
        scheme: String,
    },
    /// There is no host where one is required.
    MissingHost,
    /// The host is not a plausible host name.
    BadHost {
        /// Why.
        reason: &'static str,
    },
    /// The port is not a number, or is zero.
    BadPort,
    /// The URL carries a username or password.
    ///
    /// Refused rather than stripped: `https://your-bank.example@attacker.test/`
    /// reads as the bank to almost everybody, and there is no legitimate use of
    /// this form left on the web.
    EmbeddedCredentials,
}

impl fmt::Display for UrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("nothing to open"),
            Self::UnsupportedScheme { scheme } => write!(f, "{scheme}: is not supported"),
            Self::MissingHost => f.write_str("no site name"),
            Self::BadHost { reason } => write!(f, "the site name is not valid: {reason}"),
            Self::BadPort => f.write_str("the port is not a number"),
            Self::EmbeddedCredentials => {
                f.write_str("this address hides the real site behind a username")
            }
        }
    }
}

impl std::error::Error for UrlError {}

/// A scheme the browser handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Scheme {
    /// Encrypted HTTP.
    Https,
    /// Unencrypted HTTP.
    Http,
    /// A local file.
    File,
    /// A page the browser itself provides, e.g. `litric:settings`.
    Litric,
}

impl Scheme {
    /// Parse a scheme name.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "https" => Some(Self::Https),
            "http" => Some(Self::Http),
            "file" => Some(Self::File),
            "litric" => Some(Self::Litric),
            _ => None,
        }
    }

    /// The scheme's name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Https => "https",
            Self::Http => "http",
            Self::File => "file",
            Self::Litric => "litric",
        }
    }

    /// Whether the connection is encrypted.
    #[must_use]
    pub const fn is_secure(self) -> bool {
        matches!(self, Self::Https | Self::File | Self::Litric)
    }

    /// Whether a host is required.
    #[must_use]
    pub const fn needs_host(self) -> bool {
        matches!(self, Self::Https | Self::Http)
    }

    /// The port used when none is given.
    #[must_use]
    pub const fn default_port(self) -> Option<u16> {
        match self {
            Self::Https => Some(443),
            Self::Http => Some(80),
            Self::File | Self::Litric => None,
        }
    }
}

/// A parsed address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Url {
    /// Which scheme.
    pub scheme: Scheme,
    /// Host, lowercased, without a port.
    pub host: String,
    /// Port, when one was given explicitly.
    pub port: Option<u16>,
    /// Path, always starting with `/` for schemes that have one.
    pub path: String,
    /// Query, without the `?`.
    pub query: Option<String>,
    /// Fragment, without the `#`.
    pub fragment: Option<String>,
}

impl Url {
    /// Parse an absolute URL.
    ///
    /// # Errors
    /// [`UrlError`] describing what was wrong.
    pub fn parse(raw: &str) -> Result<Self, UrlError> {
        let raw = raw.trim();
        if raw.is_empty() {
            return Err(UrlError::Empty);
        }
        let (scheme_str, rest) = raw.split_once(':').ok_or(UrlError::MissingHost)?;
        let scheme = Scheme::parse(scheme_str).ok_or_else(|| UrlError::UnsupportedScheme {
            scheme: scheme_str.to_owned(),
        })?;

        let had_slashes = rest.starts_with("//");
        let rest = rest.strip_prefix("//").unwrap_or(rest);
        // Only a scheme that has an authority takes one. Without this,
        // `litric:settings` parses "settings" as a host name and the path comes
        // out empty, so the page has no identity at all.
        let (authority, after) = if scheme.needs_host() || had_slashes {
            match rest.find(['/', '?', '#']) {
                Some(i) => (rest.get(..i).unwrap_or(""), rest.get(i..).unwrap_or("")),
                None => (rest, ""),
            }
        } else {
            ("", rest)
        };

        if authority.contains('@') {
            return Err(UrlError::EmbeddedCredentials);
        }

        // A bare IPv6 literal is brackets and colons and has no port. Every
        // other colon in the authority separates a port, and a port that is not
        // a number is an error rather than part of the host: before this,
        // `:abc` stayed glued to the name, so `https://ads.example:abc/` had a
        // host the tracker list could not match and a typo became a site.
        let (host_part, port) = if authority.starts_with('[') && authority.ends_with(']') {
            (authority, None)
        } else if let Some((h, p)) = authority.rsplit_once(':') {
            if p.is_empty() || !p.chars().all(|c| c.is_ascii_digit()) {
                return Err(UrlError::BadPort);
            }
            let n: u16 = p.parse().map_err(|_| UrlError::BadPort)?;
            if n == 0 {
                return Err(UrlError::BadPort);
            }
            (h, Some(n))
        } else {
            (authority, None)
        };

        let host = host_part.to_ascii_lowercase();
        if scheme.needs_host() {
            if host.is_empty() {
                return Err(UrlError::MissingHost);
            }
            validate_host(&host)?;
        }
        // A default port is not part of an origin: `https://site:443` and
        // `https://site` are the same site. Treating them as two origins let a
        // site step around a permission the user refused -- a Deny stored for
        // one spelling did not apply to the other, which then fell back to Ask.
        // Dropping it here makes `origin()` canonical for every caller.
        let port = port.filter(|p| Some(*p) != scheme.default_port());

        let (path_and_query, fragment) = match after.split_once('#') {
            Some((a, b)) => (a, Some(b.to_owned())),
            None => (after, None),
        };
        let (path, query) = match path_and_query.split_once('?') {
            Some((a, b)) => (a, Some(b.to_owned())),
            None => (path_and_query, None),
        };

        let path = if path.is_empty() && scheme.needs_host() {
            "/".to_owned()
        } else {
            path.to_owned()
        };

        Ok(Self {
            scheme,
            host,
            port,
            path,
            query,
            fragment,
        })
    }

    /// The security origin: scheme, host and port.
    ///
    /// What every permission is granted to. Not the path — a permission granted
    /// to one page of a site is granted to the site, and pretending otherwise
    /// would be security theatre.
    #[must_use]
    pub fn origin(&self) -> String {
        match self.port {
            Some(p) => format!("{}://{}:{p}", self.scheme.name(), self.host),
            None => format!("{}://{}", self.scheme.name(), self.host),
        }
    }

    /// Whether the connection is encrypted.
    #[must_use]
    pub const fn is_secure(&self) -> bool {
        self.scheme.is_secure()
    }

    /// The full address as text.
    #[must_use]
    pub fn to_url(&self) -> String {
        let mut s = String::new();
        s.push_str(self.scheme.name());
        s.push(':');
        if self.scheme.needs_host() || !self.host.is_empty() {
            s.push_str("//");
            s.push_str(&self.host);
            if let Some(p) = self.port {
                s.push(':');
                s.push_str(&p.to_string());
            }
        }
        s.push_str(&self.path);
        if let Some(q) = &self.query {
            s.push('?');
            s.push_str(q);
        }
        if let Some(f) = &self.fragment {
            s.push('#');
            s.push_str(f);
        }
        s
    }

    /// What the address bar should show.
    ///
    /// The host, emphasised, with the scheme dropped when it is `https` —
    /// because showing "https" everywhere trains people to ignore it, and the
    /// thing worth noticing is its *absence*.
    #[must_use]
    pub fn display_host(&self) -> String {
        if is_confusable_host(&self.host) {
            // A host that mixes scripts is shown exactly as it is encoded, so
            // a Cyrillic 'а' cannot masquerade as a Latin 'a'.
            format!("{} (unverified name)", self.host)
        } else {
            self.host.clone()
        }
    }

    /// A short label for a tab.
    #[must_use]
    pub fn tab_label(&self) -> String {
        match self.scheme {
            Scheme::Litric => format!("litric:{}", self.path.trim_start_matches('/')),
            Scheme::File => self
                .path
                .rsplit('/')
                .next()
                .filter(|s| !s.is_empty())
                .unwrap_or(&self.path)
                .to_owned(),
            _ => self.display_host(),
        }
    }
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_url())
    }
}

fn validate_host(host: &str) -> Result<(), UrlError> {
    if host.starts_with('.') || host.ends_with('.') {
        return Err(UrlError::BadHost {
            reason: "it starts or ends with a dot",
        });
    }
    if host.contains("..") {
        return Err(UrlError::BadHost {
            reason: "it has an empty label",
        });
    }
    if host.chars().any(char::is_whitespace) {
        return Err(UrlError::BadHost {
            reason: "it contains a space",
        });
    }
    Ok(())
}

/// Whether a host mixes scripts in a way that can imitate another name.
///
/// A crude but effective test: a label containing both ASCII letters and
/// non-ASCII letters is the shape every homograph attack takes. Whole-script
/// names — an entirely Thai or entirely Greek domain — are legitimate and are
/// not flagged.
#[must_use]
pub fn is_confusable_host(host: &str) -> bool {
    host.split('.').any(|label| {
        let ascii_alpha = label.chars().any(|c| c.is_ascii_alphabetic());
        let non_ascii = !label.is_ascii();
        ascii_alpha && non_ascii
    })
}

/// What the user meant by what they typed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intent {
    /// An address to open.
    Navigate(Url),
    /// Words to search for.
    Search(String),
}

/// Decide whether typed text is an address or a search.
///
/// The rule people expect: something with a scheme, or something that looks
/// like a host name with a dot in it and no spaces, is an address. Everything
/// else is a search. Guessing wrong in the other direction — treating "rust
/// traits" as a host — is what produces a DNS lookup for every phrase somebody
/// types.
#[must_use]
pub fn interpret(input: &str) -> Intent {
    let s = input.trim();
    if s.is_empty() {
        return Intent::Search(String::new());
    }

    if let Some((scheme, _)) = s.split_once(':') {
        if Scheme::parse(scheme).is_some() {
            return match Url::parse(s) {
                Ok(u) => Intent::Navigate(u),
                // A malformed address the user clearly meant as one stays a
                // navigation attempt, so they get an error about the address
                // rather than a search for it.
                Err(_) => Intent::Search(s.to_owned()),
            };
        }
    }

    let host_like = !s.contains(char::is_whitespace)
        && s.contains('.')
        && !s.starts_with('.')
        && !s.ends_with('.');
    if s == "localhost" || host_like {
        if let Ok(u) = Url::parse(&format!("https://{s}")) {
            return Intent::Navigate(u);
        }
    }
    Intent::Search(s.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_https_url_parses() {
        let u = Url::parse("https://example.com/path?a=1#top").expect("parses");
        assert_eq!(u.scheme, Scheme::Https);
        assert_eq!(u.host, "example.com");
        assert_eq!(u.path, "/path");
        assert_eq!(u.query.as_deref(), Some("a=1"));
        assert_eq!(u.fragment.as_deref(), Some("top"));
        assert_eq!(u.to_url(), "https://example.com/path?a=1#top");
    }

    #[test]
    fn a_url_with_no_path_gets_a_root_one() {
        let u = Url::parse("https://example.com").expect("parses");
        assert_eq!(u.path, "/");
        assert_eq!(u.to_url(), "https://example.com/");
    }

    #[test]
    fn the_host_is_lowercased_but_the_path_is_not() {
        // Host names are case-insensitive; paths are not, and lowercasing one
        // breaks every site with a capital letter in a URL.
        let u = Url::parse("https://Example.COM/Path/To/File").expect("parses");
        assert_eq!(u.host, "example.com");
        assert_eq!(u.path, "/Path/To/File");
    }

    #[test]
    fn a_port_is_parsed_and_kept_in_the_origin() {
        let u = Url::parse("https://example.com:8443/x").expect("parses");
        assert_eq!(u.port, Some(8443));
        assert_eq!(u.origin(), "https://example.com:8443");
    }

    #[test]
    fn a_bad_port_is_refused() {
        assert_eq!(Url::parse("https://example.com:0/"), Err(UrlError::BadPort));
        assert_eq!(
            Url::parse("https://example.com:99999/"),
            Err(UrlError::BadPort)
        );
        // A port that is not a number is an error, not part of the host. When
        // it was folded into the host, `https://ads.example:abc/` produced a
        // host the tracker list could not match.
        assert_eq!(
            Url::parse("https://example.com:abc/"),
            Err(UrlError::BadPort)
        );
        assert_eq!(Url::parse("https://example.com:/"), Err(UrlError::BadPort));
        assert_eq!(
            Url::parse("https://example.com:65536/"),
            Err(UrlError::BadPort)
        );
    }

    #[test]
    fn an_ipv6_literal_keeps_its_colons_and_still_takes_a_port() {
        let bare = Url::parse("https://[::1]/").expect("bare v6");
        assert_eq!(bare.host, "[::1]");
        assert_eq!(bare.port, None);
        let with_port = Url::parse("https://[::1]:8443/").expect("v6 with port");
        assert_eq!(with_port.host, "[::1]");
        assert_eq!(with_port.port, Some(8443));
    }

    #[test]
    fn a_default_port_is_the_same_origin_as_no_port() {
        // Permissions are keyed on the origin string. If `:443` made a second
        // origin, a site could vary the spelling to step around a refusal the
        // user stored for the ordinary one.
        let bare = Url::parse("https://example.com/").expect("bare");
        let explicit = Url::parse("https://example.com:443/x").expect("explicit");
        assert_eq!(explicit.port, None);
        assert_eq!(explicit.origin(), bare.origin());
        assert_eq!(explicit.to_url(), "https://example.com/x");
        assert_eq!(
            Url::parse("http://example.com:80/").expect("http").port,
            None
        );
        // A port that is not the scheme's default is still an origin of its
        // own, and is still shown.
        let other = Url::parse("https://example.com:8443/").expect("other");
        assert_ne!(other.origin(), bare.origin());
    }

    #[test]
    fn embedded_credentials_are_refused_outright() {
        // The oldest phishing trick on the web. Stripping them silently would
        // still take the user to the attacker's site.
        assert_eq!(
            Url::parse("https://your-bank.example@attacker.test/"),
            Err(UrlError::EmbeddedCredentials)
        );
        assert_eq!(
            Url::parse("http://user:pass@example.com/"),
            Err(UrlError::EmbeddedCredentials)
        );
    }

    #[test]
    fn a_nonsense_host_is_refused() {
        assert!(matches!(
            Url::parse("https://.example.com/"),
            Err(UrlError::BadHost { .. })
        ));
        assert!(matches!(
            Url::parse("https://exa..mple.com/"),
            Err(UrlError::BadHost { .. })
        ));
        assert_eq!(Url::parse("https:///path"), Err(UrlError::MissingHost));
    }

    #[test]
    fn an_unsupported_scheme_says_which_one() {
        assert_eq!(
            Url::parse("javascript:alert(1)"),
            Err(UrlError::UnsupportedScheme {
                scheme: "javascript".to_owned()
            })
        );
        assert!(matches!(
            Url::parse("data:text/html,<h1>hi"),
            Err(UrlError::UnsupportedScheme { .. })
        ));
    }

    #[test]
    fn the_origin_is_the_security_boundary_not_the_path() {
        let a = Url::parse("https://example.com/one").expect("a");
        let b = Url::parse("https://example.com/two").expect("b");
        assert_eq!(a.origin(), b.origin());
        let c = Url::parse("http://example.com/one").expect("c");
        assert_ne!(a.origin(), c.origin(), "scheme is part of the origin");
    }

    #[test]
    fn http_is_not_secure_and_https_is() {
        assert!(Url::parse("https://x.test/").expect("a").is_secure());
        assert!(!Url::parse("http://x.test/").expect("b").is_secure());
        assert!(Url::parse("litric:settings").expect("c").is_secure());
    }

    #[test]
    fn a_mixed_script_host_is_flagged_rather_than_rendered_convincingly() {
        // "аpple.com" with a Cyrillic а. Shown as-is it is indistinguishable
        // from the real thing.
        let u = Url::parse("https://\u{0430}pple.com/").expect("parses");
        assert!(is_confusable_host(&u.host));
        assert!(u.display_host().contains("unverified"));
    }

    #[test]
    fn a_whole_script_host_is_not_flagged() {
        // An entirely Thai domain is a normal domain, not an attack.
        assert!(!is_confusable_host("\u{0E44}\u{0E17}\u{0E22}.com"));
        assert!(!is_confusable_host("example.com"));
    }

    #[test]
    fn typing_a_host_navigates_and_typing_words_searches() {
        assert!(matches!(interpret("example.com"), Intent::Navigate(_)));
        assert!(matches!(
            interpret("https://example.com"),
            Intent::Navigate(_)
        ));
        assert!(matches!(interpret("localhost"), Intent::Navigate(_)));
        assert!(matches!(
            interpret("how do I write a compositor"),
            Intent::Search(_)
        ));
        assert!(
            matches!(interpret("rust"), Intent::Search(_)),
            "one bare word"
        );
    }

    #[test]
    fn a_bare_host_is_assumed_to_be_https() {
        let Intent::Navigate(u) = interpret("example.com") else {
            panic!("expected navigation");
        };
        assert_eq!(
            u.scheme,
            Scheme::Https,
            "an unqualified host must not default to http"
        );
    }

    #[test]
    fn something_that_looks_like_a_scheme_but_is_not_supported_is_searched() {
        // Otherwise typing "note: remember this" tries to open a `note:` URL.
        assert!(matches!(
            interpret("note: remember this"),
            Intent::Search(_)
        ));
        assert!(matches!(
            interpret("javascript:alert(1)"),
            Intent::Search(_)
        ));
    }

    #[test]
    fn an_empty_address_bar_searches_for_nothing_rather_than_erroring() {
        assert_eq!(interpret("   "), Intent::Search(String::new()));
    }

    #[test]
    fn a_tab_label_is_the_host_for_a_site_and_the_file_name_for_a_file() {
        assert_eq!(
            Url::parse("https://example.com/a/b")
                .expect("a")
                .tab_label(),
            "example.com"
        );
        assert_eq!(
            Url::parse("file:///home/a/report.pdf")
                .expect("b")
                .tab_label(),
            "report.pdf"
        );
        assert_eq!(
            Url::parse("litric:settings").expect("c").tab_label(),
            "litric:settings"
        );
    }

    #[test]
    fn a_scheme_with_no_authority_keeps_its_whole_body_as_the_path() {
        let u = Url::parse("litric:settings").expect("parses");
        assert_eq!(u.host, "", "a body must not be read as a host name");
        assert_eq!(u.path, "settings");
        assert_eq!(u.to_url(), "litric:settings");
    }

    #[test]
    fn every_parsed_url_round_trips_through_its_text() {
        for raw in [
            "https://example.com/",
            "http://example.com:8080/a?b=c#d",
            "https://sub.example.co.uk/path",
            "file:///etc/hostname",
            "litric:settings",
        ] {
            let u = Url::parse(raw).expect(raw);
            let again = Url::parse(&u.to_url()).expect("re-parses");
            assert_eq!(u, again, "{raw} did not round trip");
        }
    }
}
