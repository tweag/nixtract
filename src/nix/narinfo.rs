#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]

/// `NarInfo` is a struct that represents the information about a Nix Archive (nar).
pub struct NarInfo {
    /// The path to the store where the nar is located.
    pub store_path: String,
    /// The URL where the nar can be downloaded.
    pub url: String,
    /// The compression method used on the nar.
    pub compression: String,
    /// The hash of the file.
    pub file_hash: String,
    /// The size of the file in bytes.
    pub file_size: usize,
    /// The hash of the nar.
    pub nar_hash: String,
    /// The size of the nar in bytes.
    pub nar_size: usize,
    /// The deriver of the nar, if any.
    pub deriver: Option<String>,
    /// The system for which the nar is intended, if specified.
    pub system: Option<String>,
    /// The references of the nar.
    pub references: Vec<String>,
    /// The signature of the nar.
    pub sig: String,
    /// The content addressable storage identifier of the nar, if any.
    pub ca: Option<String>,
}

impl NarInfo {
    /// Fetches the narinfo file for a given output path from a list of servers.
    ///
    /// # Arguments
    ///
    /// * `output_path` - The output path of the narinfo file.
    /// * `servers` - A slice of server URLs to fetch the narinfo file from.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing an `Option` of `NarInfo` if the narinfo file is found, or `None` if it is not found.
    ///
    /// # Errors
    ///
    /// Returns an error if the output path is invalid or if there is an error fetching or parsing the narinfo file.
    ///
    /// # Example
    ///
    /// ```
    /// use nixtract::narinfo::NarInfo;
    ///
    /// let output_path = "/nix/store/abc123";
    /// let servers = vec!["server1.example.com".to_string(), "server2.example.com".to_string()];
    ///
    /// match NarInfo::fetch(output_path, &servers) {
    ///     Ok(Some(narinfo)) => {
    ///         // Narinfo file found, do something with it
    ///     },
    ///     Ok(None) => {
    ///         // Narinfo file not found
    ///     },
    ///     Err(err) => {
    ///         // Error fetching or parsing narinfo file
    ///     }
    /// }
    /// ```
    pub fn fetch(output_path: &str, servers: &[String]) -> crate::error::Result<Option<Self>> {
        // Strip the /nix/store prefix, and everything after the first -
        let hash = output_path
            .strip_prefix("/nix/store/")
            .ok_or_else(|| crate::error::Error::NarInfoInvalidPath(output_path.to_string()))?
            .split('-')
            .next()
            .ok_or_else(|| crate::error::Error::NarInfoInvalidPath(output_path.to_string()))?;

        for server in servers {
            let url = format!("https://{}/{}.narinfo", server, hash);

            log::info!("Fetching narinfo from {}", url);
            if let Ok(response) = reqwest::blocking::get(&url) {
                if response.status().is_success() {
                    let narinfo = response.text()?;
                    return Ok(Some(Self::parse(&narinfo)?));
                } else {
                    log::warn!("Cache responded with error code: {}", response.status());
                }
            }
        }

        Ok(None)
    }

    /// Parses a `narinfo` string into a `NarInfo` struct.
    ///
    /// This function takes a `narinfo` string and parses it into a `NarInfo` struct.
    /// It iterates over each line in the `narinfo` string, splitting it into a key-value pair.
    /// The key is then matched to a corresponding field in the `NarInfo` struct.
    /// If a key does not match any field, an error is returned.
    /// If a required field is missing from the `narinfo` string, an error is returned.
    ///
    /// # Arguments
    ///
    /// * `narinfo` - A string slice that holds the `narinfo` data.
    ///
    /// # Returns
    ///
    /// * `Result<NarInfo, Error>` - Returns a `NarInfo` struct if parsing is successful, otherwise returns an `Error`.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * A line in the `narinfo` string does not contain a ':' delimiter.
    /// * The `narinfo` string contains an unknown key.
    /// * A required field is missing from the `narinfo` string.
    pub fn parse(narinfo: &str) -> crate::error::Result<Self> {
        let mut store_path = None;
        let mut url = None;
        let mut compression = None;
        let mut file_hash = None;
        let mut file_size = None;
        let mut nar_hash = None;
        let mut nar_size = None;
        let mut deriver = None;
        let mut system = None;
        let mut references = Vec::new();
        let mut sig = None;
        let mut ca = None;

        for line in narinfo.lines() {
            let (key, value) = line
                .split_once(':')
                .ok_or_else(|| crate::error::Error::NarInfoNoDelimiter(line.to_string()))
                .map(|(key, value)| (key.trim(), value.trim()))?;

            match key {
                "StorePath" => store_path = Some(value.to_string()),
                "URL" => url = Some(value.to_string()),
                "Compression" => compression = Some(value.to_string()),
                "FileHash" => file_hash = Some(value.to_string()),
                "FileSize" => file_size = Some(value.parse()?),
                "NarHash" => nar_hash = Some(value.to_string()),
                "NarSize" => nar_size = Some(value.parse()?),
                "Deriver" => deriver = Some(value.to_string()),
                "System" => system = Some(value.to_string()),
                "References" => references = value.split(' ').map(|s| s.to_string()).collect(),
                "Sig" => sig = Some(value.to_string()),
                "CA" => ca = Some(value.to_string()),
                _ => {
                    log::warn!(
                        "Found an unknown key while parsing a .narinfo file ({}). Please report this issue to github.com/tweag/nixtract",
                        key
                    );
                }
            }
        }

        Ok(NarInfo {
            store_path: store_path
                .ok_or_else(|| crate::error::Error::NarInfoMissingField("StorePath".to_string()))?,
            url: url.ok_or_else(|| crate::error::Error::NarInfoMissingField("URL".to_string()))?,
            compression: compression.ok_or_else(|| {
                crate::error::Error::NarInfoMissingField("Compression".to_string())
            })?,
            file_hash: file_hash
                .ok_or_else(|| crate::error::Error::NarInfoMissingField("FileHash".to_string()))?,
            file_size: file_size
                .ok_or_else(|| crate::error::Error::NarInfoMissingField("FileSize".to_string()))?,
            nar_hash: nar_hash
                .ok_or_else(|| crate::error::Error::NarInfoMissingField("NarHash".to_string()))?,
            nar_size: nar_size
                .ok_or_else(|| crate::error::Error::NarInfoMissingField("NarSize".to_string()))?,
            deriver,
            system,
            references,
            sig: sig.ok_or_else(|| crate::error::Error::NarInfoMissingField("Sig".to_string()))?,
            ca,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch() {
        let result = NarInfo::fetch(
            "/nix/store/cg8a576pz2yfc1wbhxm1zy4x7lrk8pix-hello-2.12.1",
            &["cache.nixos.org".to_owned()],
        )
        .unwrap();

        let expected = NarInfo {
            store_path: "/nix/store/cg8a576pz2yfc1wbhxm1zy4x7lrk8pix-hello-2.12.1".to_string(),
            url: "nar/1wjh5hhqfi30fx8pqi0901c9n035qbwsv1rmizvmpydva2lpri2g.nar.xz".to_string(),
            compression: "xz".to_string(),
            file_hash: "sha256:1wjh5hhqfi30fx8pqi0901c9n035qbwsv1rmizvmpydva2lpri2g".to_string(),
            file_size: 50184,
            nar_hash: "sha256:0scilhfg9qij3wiz1irrln5nb5nk3nxfkns6yqfh2kvbaixywv26".to_string(),
            nar_size: 226552,
            deriver: Some("57677sld6ja212hkv1gh8bdm0amnk1hz-hello-2.12.1.drv".to_string()),
            system: None,
            references: vec![
                "cg8a576pz2yfc1wbhxm1zy4x7lrk8pix-hello-2.12.1".to_string(),
                "gqghjch4p1s69sv4mcjksb2kb65rwqjy-glibc-2.38-23".to_string(),
            ],
            sig: "cache.nixos.org-1:WzRvexDdRP62D8j/4rAk73vAc4gUtAN7qpZesuRc74+My03WcvWxg/LUztmWikOaMqJQJMvB1ria6AIX30yrDw==".to_string(),
            ca: None,
        };

        pretty_assertions::assert_eq!(result, Some(expected));
    }

    #[test]
    fn test_parse() {
        let narinfo = "StorePath: /nix/store/cg8a576pz2yfc1wbhxm1zy4x7lrk8pix-hello-2.12.1
URL: nar/1wjh5hhqfi30fx8pqi0901c9n035qbwsv1rmizvmpydva2lpri2g.nar.xz
Compression: xz
FileHash: sha256:1wjh5hhqfi30fx8pqi0901c9n035qbwsv1rmizvmpydva2lpri2g
FileSize: 50184
NarHash: sha256:0scilhfg9qij3wiz1irrln5nb5nk3nxfkns6yqfh2kvbaixywv26
NarSize: 226552
References: cg8a576pz2yfc1wbhxm1zy4x7lrk8pix-hello-2.12.1 gqghjch4p1s69sv4mcjksb2kb65rwqjy-glibc-2.38-23
Deriver: 57677sld6ja212hkv1gh8bdm0amnk1hz-hello-2.12.1.drv
Sig: cache.nixos.org-1:WzRvexDdRP62D8j/4rAk73vAc4gUtAN7qpZesuRc74+My03WcvWxg/LUztmWikOaMqJQJMvB1ria6AIX30yrDw==
";

        let expected = NarInfo {
            store_path: "/nix/store/cg8a576pz2yfc1wbhxm1zy4x7lrk8pix-hello-2.12.1".to_string(),
            url: "nar/1wjh5hhqfi30fx8pqi0901c9n035qbwsv1rmizvmpydva2lpri2g.nar.xz".to_string(),
            compression: "xz".to_string(),
            file_hash: "sha256:1wjh5hhqfi30fx8pqi0901c9n035qbwsv1rmizvmpydva2lpri2g".to_string(),
            file_size: 50184,
            nar_hash: "sha256:0scilhfg9qij3wiz1irrln5nb5nk3nxfkns6yqfh2kvbaixywv26".to_string(),
            nar_size: 226552,
            deriver: Some("57677sld6ja212hkv1gh8bdm0amnk1hz-hello-2.12.1.drv".to_string()),
            system: None,
            references: vec![
                "cg8a576pz2yfc1wbhxm1zy4x7lrk8pix-hello-2.12.1".to_string(),
                "gqghjch4p1s69sv4mcjksb2kb65rwqjy-glibc-2.38-23".to_string(),
            ],
            sig: "cache.nixos.org-1:WzRvexDdRP62D8j/4rAk73vAc4gUtAN7qpZesuRc74+My03WcvWxg/LUztmWikOaMqJQJMvB1ria6AIX30yrDw==".to_string(),
            ca: None,
        };

        let result = NarInfo::parse(narinfo).unwrap();
        pretty_assertions::assert_eq!(result, expected);
    }
}
