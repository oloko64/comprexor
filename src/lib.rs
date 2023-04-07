use flate2::write::GzEncoder;
use humansize::{make_format, DECIMAL};
use owo_colors::OwoColorize;
use std::{
    collections::hash_map::DefaultHasher,
    fmt,
    hash::Hasher,
    io::{copy, BufReader},
    path::PathBuf,
};
use tar::Archive;

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

pub struct Compressor<'a> {
    input: &'a str,
    output: &'a str,
}

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
    /// use compress_rs::Extractor;
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
    /// use compress_rs::Extractor;
    ///
    /// let extractor = Extractor::new("./compacted-archive.tar.gz", "./output-folder-or-file");
    /// extractor.extract().unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if the input file is not a valid gzip file or something goes wrong while decompressing
    pub fn extract(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "Decompressing {} to {}\n",
            self.input.green(),
            self.output.green()
        );

        self.extract_internal()?;

        Ok(())
    }

    fn extract_internal(&self) -> Result<(), Box<dyn std::error::Error>> {
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

        let formatter = make_format(DECIMAL);
        println!("Input size: {}", formatter(input_size).green());
        println!("Output size: {}", formatter(output_size).green());
        println!(
            "Compression ratio: {:.3}",
            (output_size as f64 / input_size as f64).green()
        );

        Ok(())
    }
}

impl<'a> Compressor<'a> {
    #[must_use]
    /// Creates a new compressor with the given input and output
    ///
    /// # Example
    ///
    /// ```
    /// use compress_rs::Compressor;
    ///
    /// let compressor = Compressor::new("./folder-or-file-to-compress", "./compacted-archive.tar.gz");
    /// compressor.compress().unwrap();
    /// ```
    pub fn new(input: &'a str, output: &'a str) -> Self {
        Self { input, output }
    }

    /// Compress the input file or folder to the output location
    ///
    /// # Example
    ///
    /// ```
    /// use compress_rs::Compressor;
    ///
    /// let compressor = Compressor::new("./folder-or-file-to-compress", "./compacted-archive.tar.gz");
    /// compressor.compress().unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if the input file is not a valid gzip file or something goes wrong while compressing
    pub fn compress(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "Compressing {} to {}\n",
            self.input.green(),
            self.output.green()
        );
        self.compress_with_tar()?;

        Ok(())
    }

    fn compress_with_tar(&self) -> Result<(), Box<dyn std::error::Error>> {
        let tar_temp = Self::get_hashed_file_in_temp(self.input);

        let files_to_append = if std::fs::metadata(self.input)?.is_dir() {
            println!("Getting files from directory...");
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

        println!("{}", "Tar file created, compressing...\n".green());
        self.compress_internal(tar_temp.to_str().ok_or("Invalid path")?)?;

        std::fs::remove_file(tar_temp)?;

        Ok(())
    }

    fn compress_internal<T>(&self, input: T) -> Result<(), Box<dyn std::error::Error>>
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

        let formatter = make_format(DECIMAL);
        println!("Input size: {}", formatter(input_size).green());
        println!("Output size: {}", formatter(output_size).green());
        println!(
            "Compression ratio: {:.3}",
            (input_size as f64 / output_size as f64).green()
        );

        Ok(())
    }
}
