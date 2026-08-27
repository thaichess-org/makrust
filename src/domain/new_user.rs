use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{Error as Argon2Error, SaltString, rand_core::OsRng},
};
use validator::ValidateEmail;

pub struct NewUser {
    pub username: Username,
    pub email: UserEmail,
    pub password_hash: String,
}

pub enum PasswordError {
    HashingError(Argon2Error),
    InvalidPassword,
}

pub enum NewUserError {
    UsernameParsingError,
    EmailParsingError,
    PasswordError(PasswordError),
}

impl From<Argon2Error> for PasswordError {
    fn from(err: Argon2Error) -> Self {
        PasswordError::HashingError(err)
    }
}

impl From<PasswordError> for NewUserError {
    fn from(err: PasswordError) -> Self {
        NewUserError::PasswordError(err)
    }
}

impl NewUser {
    pub fn new(username: String, email: String, password: String) -> Result<Self, NewUserError> {
        let username = Username::parse(username)?;
        let email = UserEmail::parse(email)?;
        let password = Password::parse(password)?;
        let password_hash = password.hash()?;

        Ok(NewUser {
            username,
            email,
            password_hash,
        })
    }
}

pub struct Username(String);

impl AsRef<str> for Username {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Username {
    pub fn parse(s: String) -> Result<Username, NewUserError> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.len() > 50;

        let forbidden_characters = [
            '@', '#', '$', '%', '&', ',', '(', ')', '-', '+', '=', '{', '}', '[', ']', '<', '>',
            ';', '.', '^', '*', '-', ' ',
        ];
        let contains_forbidden_characters =
            s.trim().chars().any(|c| forbidden_characters.contains(&c));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            Err(NewUserError::UsernameParsingError)
        } else {
            Ok(Self(s))
        }
    }
}

#[cfg(test)]
mod username_tests {
    use super::*;
    #[test]
    fn a_50_characters_username_is_valid() {
        let name = "a".repeat(50);
        assert!(Username::parse(name).is_ok());
    }

    #[test]
    fn a_username_bigger_than_50_characters_is_not_valid() {
        let name = "a".repeat(51);
        assert!(Username::parse(name).is_err())
    }

    #[test]
    fn whitespace_username_is_not_valid() {
        let name = " ".to_string();
        assert!(Username::parse(name).is_err())
    }

    #[test]
    fn empty_string_is_not_valid() {
        let name = "".to_string();
        assert!(Username::parse(name).is_err())
    }

    #[test]
    fn username_with_forbidden_characters_is_not_valid() {
        let name = "jonager@".to_string();
        assert!(Username::parse(name).is_err())
    }

    #[test]
    fn username_with_space_is_not_valid() {
        let name = "thaichess master".to_string();
        assert!(Username::parse(name).is_err())
    }

    #[test]
    fn valid_username_is_parsed_successfully() {
        let name = "thaiChessMaster27".to_string();
        assert!(Username::parse(name).is_ok())
    }
}

pub struct UserEmail(String);

impl AsRef<str> for UserEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl UserEmail {
    pub fn parse(s: String) -> Result<UserEmail, NewUserError> {
        if ValidateEmail::validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(NewUserError::EmailParsingError)
        }
    }
}

#[cfg(test)]
mod email_tests {
    use super::*;

    #[test]
    fn valid_emails_are_parsed_sucessfully() {
        let valid_emails: Vec<&str> = vec![
            "email@example.com",
            "firstname.lastname@example.com",
            "email@subdomain.example.com",
            "firstname+lastname@example.com",
            "email@123.123.123.123",
            "email@[123.123.123.123]",
            "1234567890@example.com",
            "email@example-one.com",
            "_______@example.com",
            "email@example.name",
            "email@example.museum",
            "email@example.co.jp",
            "firstname-lastname@example.com",
        ];

        for email in valid_emails {
            assert!(UserEmail::parse(email.to_string()).is_ok());
        }
    }

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert!(UserEmail::parse(email).is_err());
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursuladomain.com".to_string();
        assert!(UserEmail::parse(email).is_err());
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        assert!(UserEmail::parse(email).is_err());
    }
}

pub struct Password(String);

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Password {
    pub fn parse(s: String) -> Result<Password, PasswordError> {
        if s.trim().len() < 8 || s.trim().len() > 64 || s.trim().contains(char::is_whitespace) {
            return Err(PasswordError::InvalidPassword);
        } else {
            Ok(Self(s))
        }
    }

    // TODO: test these 2 functions
    pub fn hash(&self) -> Result<String, PasswordError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let passsword_hash = argon2
            .hash_password(self.as_ref().as_bytes(), &salt)
            .map_err(PasswordError::HashingError)?
            .to_string();
        Ok(passsword_hash)
    }

    pub fn verify(password: &String, password_hash: &String) -> Result<bool, PasswordError> {
        let parsed_hash = PasswordHash::new(&password_hash).map_err(PasswordError::HashingError)?;
        let argon2 = Argon2::default();
        argon2
            .verify_password(&password.as_bytes(), &parsed_hash)
            .map_err(|e| match e {
                Argon2Error::Password => PasswordError::InvalidPassword,
                e => PasswordError::HashingError(e),
            })
            .map(|_| true)
    }
}

#[cfg(test)]
mod password_tests {
    use super::*;

    #[test]
    fn short_password_is_not_valid() {
        let password = "short".to_string();
        assert!(Password::parse(password).is_err());
    }

    #[test]
    fn long_password_is_not_valid() {
        let password = "toolong".repeat(10);
        assert!(Password::parse(password).is_err());
    }

    #[test]
    fn space_is_not_valid() {
        let password = "this has spaces".to_string();
        assert!(Password::parse(password).is_err());
    }

    #[test]
    fn password_is_valid() {
        let password = "thisPasswordI$Valid".to_string();
        assert!(Password::parse(password).is_ok());
    }
}
