use flate2::write::GzEncoder;
use humansize::{make_format, DECIMAL};
use std::{
    collections::hash_map::DefaultHasher,
    fmt,
    hash::Hasher,
    io::{copy, BufReader},
    path::PathBuf,
};
use tar::Archive;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ArchiveData {
    input_size: u64,
    output_size: u64,
    ratio: f64,
}

impl ArchiveData {
    /// Get the input size without formatting
    #[must_use]
    pub fn input_size(&self) -> u64 {
        self.input_size
    }

    /// Get the input size in a human readable format
    ///
    /// # Example
    ///
    /// ```
    /// use comprexor::ArchiveData;
    ///
    /// let archive_data = ArchiveData {
    ///    input_size: 1000,
    ///    output_size: 1000,
    ///    ratio: 1.0,
    /// };
    ///
    /// assert_eq!(archive_data.input_size_formatted(), "1.0 kB");
    /// ```
    #[must_use]
    pub fn input_size_formatted(&self) -> String {
        let formatter = make_format(DECIMAL);
        formatter(self.input_size)
    }

    /// Get the output size without formatting
    #[must_use]
    pub fn output_size(&self) -> u64 {
        self.output_size
    }

    /// Get the output size in a human readable format
    ///
    /// # Example
    ///
    /// ```
    /// use comprexor::ArchiveData;
    ///
    /// let archive_data = ArchiveData {
    ///   input_size: 1000,
    ///   output_size: 1000,
    ///   ratio: 1.0,
    /// };
    ///
    /// assert_eq!(archive_data.output_size_formatted(), "1.0 kB");
    /// ```
    #[must_use]
    pub fn output_size_formatted(&self) -> String {
        let formatter = make_format(DECIMAL);
        formatter(self.output_size)
    }

    /// Get the ratio without formatting
    #[must_use]
    pub fn ratio(&self) -> f64 {
        self.ratio
    }

    /// Get the ratio formatted to the given number of decimals
    ///
    /// # Example
    ///
    /// ```
    /// use comprexor::ArchiveData;
    ///
    /// let archive_data = ArchiveData {
    ///     input_size: 1000,
    ///     output_size: 1000,
    ///     ratio: 1.0,
    /// };
    ///
    /// assert_eq!(archive_data.ratio_formatted(5), "1.00000");
    /// assert_eq!(archive_data.ratio_formatted(2), "1.00");
    /// ```
    #[must_use]
    pub fn ratio_formatted(&self, num_decimals: u8) -> String {
        format!(
            "{:.decimals$}",
            self.ratio,
            decimals = num_decimals as usize
        )
    }
}

trait ArchiveExt {
    fn get_hashed_file_in_temp(input: &str) -> PathBuf {
        let random_f64 = rand::random::<f64>();
        let temp_dir = std::env::temp_dir();
        let hashed_name = Self::gen_hashed_name(&format!("{input}{random_f64}"));
        temp_dir.join(hashed_name)
    }

    fn gen_hashed_name<T>(input: &T) -> String
    where
        T: std::hash::Hash + fmt::Display,
    {
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        format!("{}-{}", hasher.finish(), "tar-gen.tar")
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash)]
pub struct Compressor<'a> {
    input: &'a str,
    output: &'a str,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash)]
pub struct Extractor<'a> {
    input: &'a str,
    output: &'a str,
}

impl<'a> ArchiveExt for Compressor<'a> {}
impl<'a> ArchiveExt for Extractor<'a> {}

impl<'a> Extractor<'a> {
    #[must_use]
    /// Create a new extractor with the given input and output
    ///
    /// # Example
    /// ```
    /// use comprexor::Extractor;
    ///
    /// let extractor = Extractor::new("./compacted-archive.tar.gz", "./output-folder-or-file");
    /// extractor.extract().unwrap();
    /// ```
    pub fn new(input: &'a str, output: &'a str) -> Extractor<'a> {
        Self { input, output }
    }

    /// Decompress the input file to the output file
    ///
    /// # Example
    ///
    /// ```
    /// use comprexor::Extractor;
    ///
    /// let extractor = Extractor::new("./compacted-archive.tar.gz", "./output-folder-or-file");
    /// extractor.extract().unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if the input file is not a valid gzip file or something goes wrong while decompressing
    pub fn extract(&self) -> Result<ArchiveData, std::io::Error> {
        let archive_data = self.extract_internal()?;
        Ok(archive_data)
    }

    fn extract_internal(&self) -> Result<ArchiveData, std::io::Error> {
        let tar_temp = Self::get_hashed_file_in_temp(self.input);
        let input_file = BufReader::new(std::fs::File::open(self.input)?);
        let input_size = std::fs::metadata(self.input)?.len();
        let mut output_file = std::fs::File::create(&tar_temp)?;

        let mut decoder = flate2::read::GzDecoder::new(input_file);
        copy(&mut decoder, &mut output_file)?;
        let output_size = std::fs::metadata(&tar_temp)?.len();

        let file = std::fs::File::open(&tar_temp)?;
        let mut archive = Archive::new(file);
        archive.unpack(self.output)?;

        std::fs::remove_file(tar_temp)?;

        Ok(ArchiveData {
            input_size,
            output_size,
            ratio: output_size as f64 / input_size as f64,
        })
    }
}

impl<'a> Compressor<'a> {
    #[must_use]
    /// Creates a new compressor with the given input and output
    ///
    /// # Example
    ///
    /// ```
    /// use comprexor::Compressor;
    ///
    /// let compressor = Compressor::new("./folder-or-file-to-compress", "./compacted-archive.tar.gz");
    /// compressor.compress().unwrap();
    /// ```
    pub fn new(input: &'a str, output: &'a str) -> Compressor<'a> {
        Self { input, output }
    }

    /// Compress the input file or folder to the output location
    ///
    /// # Example
    ///
    /// ```
    /// use comprexor::Compressor;
    ///
    /// let compressor = Compressor::new("./folder-or-file-to-compress", "./compacted-archive.tar.gz");
    /// compressor.compress().unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if the input file is not a valid gzip file or something goes wrong while compressing
    pub fn compress(&self) -> Result<ArchiveData, std::io::Error> {
        let archive_data = self.compress_with_tar()?;

        Ok(archive_data)
    }

    fn compress_with_tar(&self) -> Result<ArchiveData, std::io::Error> {
        let tar_temp = Self::get_hashed_file_in_temp(self.input);

        let files_to_append = if std::fs::metadata(self.input)?.is_dir() {
            walkdir::WalkDir::new(self.input)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
                .map(|e| e.path().to_str().unwrap().to_string())
                .collect::<Vec<String>>()
        } else {
            vec![self.input.to_string()]
        };

        let file_tar = std::fs::File::create(&tar_temp)?;
        let mut tar = tar::Builder::new(file_tar);

        for file in files_to_append {
            tar.append_path(file)?;
        }

        tar.finish()?;

        let archive_data =
            self.compress_internal(tar_temp.to_str().ok_or(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Could not convert tar temp file to str",
            ))?)?;

        std::fs::remove_file(tar_temp)?;

        Ok(archive_data)
    }

    fn compress_internal<T>(&self, input: T) -> Result<ArchiveData, std::io::Error>
    where
        T: AsRef<str>,
    {
        let mut input_file = BufReader::new(std::fs::File::open(input.as_ref())?);
        let input_size = std::fs::metadata(input.as_ref())?.len();
        let output_file = std::fs::File::create(self.output)?;

        let mut encoder = GzEncoder::new(output_file, flate2::Compression::best());
        copy(&mut input_file, &mut encoder)?;
        encoder.finish()?;
        let output_size = std::fs::metadata(self.output)?.len();

        Ok(ArchiveData {
            input_size,
            output_size,
            ratio: input_size as f64 / output_size as f64,
        })
    }
}
