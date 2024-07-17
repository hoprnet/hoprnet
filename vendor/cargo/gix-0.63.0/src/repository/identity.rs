use std::time::SystemTime;

use crate::{
    bstr::BString,
    config,
    config::tree::{gitoxide, keys, Author, Committer, Key, User},
};

/// Identity handling.
///
/// # Deviation
///
/// There is no notion of a default user like in git, and instead failing to provide a user
/// is fatal. That way, we enforce correctness and force application developers to take care
/// of this issue which can be done in various ways, for instance by setting
/// `gitoxide.committer.nameFallback` and similar.
impl crate::Repository {
    /// Return the committer as configured by this repository, which is determined by…
    ///
    /// * …the git configuration `committer.name|email`…
    /// * …the `GIT_COMMITTER_(NAME|EMAIL|DATE)` environment variables…
    /// * …the configuration for `user.name|email` as fallback…
    ///
    /// …and in that order, or `None` if no committer name or email was configured, or `Some(Err(…))`
    /// if the committer date could not be parsed.
    ///
    /// # Note
    ///
    /// The values are cached when the repository is instantiated.
    pub fn committer(&self) -> Option<Result<gix_actor::SignatureRef<'_>, config::time::Error>> {
        let p = self.config.personas();

        Ok(gix_actor::SignatureRef {
            name: p.committer.name.as_ref().or(p.user.name.as_ref()).map(AsRef::as_ref)?,
            email: p
                .committer
                .email
                .as_ref()
                .or(p.user.email.as_ref())
                .map(AsRef::as_ref)?,
            time: match extract_time_or_default(p.committer.time.as_ref(), &gitoxide::Commit::COMMITTER_DATE) {
                Ok(t) => t,
                Err(err) => return Some(Err(err)),
            },
        })
        .into()
    }

    /// Return the author as configured by this repository, which is determined by…
    ///
    /// * …the git configuration `author.name|email`…
    /// * …the `GIT_AUTHOR_(NAME|EMAIL|DATE)` environment variables…
    /// * …the configuration for `user.name|email` as fallback…
    ///
    /// …and in that order, or `None` if there was nothing configured.
    ///
    /// # Note
    ///
    /// The values are cached when the repository is instantiated.
    pub fn author(&self) -> Option<Result<gix_actor::SignatureRef<'_>, config::time::Error>> {
        let p = self.config.personas();

        Ok(gix_actor::SignatureRef {
            name: p.author.name.as_ref().or(p.user.name.as_ref()).map(AsRef::as_ref)?,
            email: p.author.email.as_ref().or(p.user.email.as_ref()).map(AsRef::as_ref)?,
            time: match extract_time_or_default(p.author.time.as_ref(), &gitoxide::Commit::AUTHOR_DATE) {
                Ok(t) => t,
                Err(err) => return Some(Err(err)),
            },
        })
        .into()
    }
}

fn extract_time_or_default(
    time: Option<&Result<gix_date::Time, gix_date::parse::Error>>,
    config_key: &'static keys::Time,
) -> Result<gix_date::Time, config::time::Error> {
    match time {
        Some(Ok(t)) => Ok(*t),
        None => Ok(gix_date::Time::now_local_or_utc()),
        Some(Err(err)) => Err(config::time::Error::from(config_key).with_source(err.clone())),
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Entity {
    pub name: Option<BString>,
    pub email: Option<BString>,
    /// A time parsed from an environment variable, handling potential errors is delayed.
    pub time: Option<Result<gix_date::Time, gix_date::parse::Error>>,
}

#[derive(Debug, Clone)]
pub(crate) struct Personas {
    user: Entity,
    committer: Entity,
    author: Entity,
}

impl Personas {
    pub fn from_config_and_env(config: &gix_config::File<'_>) -> Self {
        fn entity_in_section(
            config: &gix_config::File<'_>,
            name_key: &keys::Any,
            email_key: &keys::Any,
            fallback: Option<(&keys::Any, &keys::Any)>,
        ) -> (Option<BString>, Option<BString>) {
            let fallback = fallback.and_then(|(name_key, email_key)| {
                debug_assert_eq!(name_key.section.name(), email_key.section.name());
                config
                    .section("gitoxide", Some(name_key.section.name().into()))
                    .ok()
                    .map(|section| (section, name_key, email_key))
            });
            (
                config
                    .string(name_key.section.name(), None, name_key.name)
                    .or_else(|| fallback.as_ref().and_then(|(s, name_key, _)| s.value(name_key.name)))
                    .map(std::borrow::Cow::into_owned),
                config
                    .string(email_key.section.name(), None, email_key.name)
                    .or_else(|| fallback.as_ref().and_then(|(s, _, email_key)| s.value(email_key.name)))
                    .map(std::borrow::Cow::into_owned),
            )
        }
        let now = SystemTime::now();
        let parse_date = |key: &str, date: &keys::Time| -> Option<Result<gix_date::Time, gix_date::parse::Error>> {
            debug_assert_eq!(
                key,
                date.logical_name(),
                "BUG: drift of expected name and actual name of the key (we hardcode it to save an allocation)"
            );
            config
                .string_by_key(key)
                .map(|time| date.try_into_time(time, now.into()))
        };

        let fallback = (
            &gitoxide::Committer::NAME_FALLBACK,
            &gitoxide::Committer::EMAIL_FALLBACK,
        );
        let (committer_name, committer_email) =
            entity_in_section(config, &Committer::NAME, &Committer::EMAIL, Some(fallback));
        let fallback = (&gitoxide::Author::NAME_FALLBACK, &gitoxide::Author::EMAIL_FALLBACK);
        let (author_name, author_email) = entity_in_section(config, &Author::NAME, &Author::EMAIL, Some(fallback));
        let (user_name, mut user_email) = entity_in_section(config, &User::NAME, &User::EMAIL, None);

        let committer_date = parse_date("gitoxide.commit.committerDate", &gitoxide::Commit::COMMITTER_DATE);
        let author_date = parse_date("gitoxide.commit.authorDate", &gitoxide::Commit::AUTHOR_DATE);

        user_email = user_email.or_else(|| {
            config
                .string_by_key(gitoxide::User::EMAIL_FALLBACK.logical_name().as_str())
                .map(std::borrow::Cow::into_owned)
        });
        Personas {
            user: Entity {
                name: user_name,
                email: user_email,
                time: None,
            },
            committer: Entity {
                name: committer_name,
                email: committer_email,
                time: committer_date,
            },
            author: Entity {
                name: author_name,
                email: author_email,
                time: author_date,
            },
        }
    }
}
