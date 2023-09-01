use core_network::{KeyProvider, SecioError, SecioKeyPair};

use protocol::async_trait;

#[derive(Clone)]
pub(crate) enum KeyP<K: KeyProvider> {
    Custom(K),
    Default(SecioKeyPair),
}

#[async_trait]
impl<K> KeyProvider for KeyP<K>
where
    K: KeyProvider,
{
    type Error = SecioError;

    async fn sign_ecdsa_async<T: AsRef<[u8]> + Send>(
        &self,
        message: T,
    ) -> Result<Vec<u8>, Self::Error> {
        match self {
            KeyP::Custom(k) => k.sign_ecdsa_async(message).await.map_err(Into::into),
            KeyP::Default(k) => k.sign_ecdsa_async(message).await,
        }
    }

    /// Constructs a signature for `msg` using the secret key `sk`
    fn sign_ecdsa<T: AsRef<[u8]>>(&self, message: T) -> Result<Vec<u8>, Self::Error> {
        match self {
            KeyP::Custom(k) => k.sign_ecdsa(message).map_err(Into::into),
            KeyP::Default(k) => k.sign_ecdsa(message),
        }
    }

    /// Creates a new public key from the [`KeyProvider`].
    fn pubkey(&self) -> Vec<u8> {
        match self {
            KeyP::Custom(k) => k.pubkey(),
            KeyP::Default(k) => k.pubkey(),
        }
    }

    /// Checks that `sig` is a valid ECDSA signature for `msg` using the
    /// pubkey.
    fn verify_ecdsa<P, T, F>(&self, pubkey: P, message: T, signature: F) -> bool
    where
        P: AsRef<[u8]>,
        T: AsRef<[u8]>,
        F: AsRef<[u8]>,
    {
        match self {
            KeyP::Custom(k) => k.verify_ecdsa(pubkey, message, signature),
            KeyP::Default(k) => k.verify_ecdsa(pubkey, message, signature),
        }
    }
}
