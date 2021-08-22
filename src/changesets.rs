use super::*;
use anyhow::{bail, ensure};
use bzip2::read::MultiBzDecoder;
use quick_xml::events::Event;
use std::fs::*;
use std::io::{BufReader, Read};

#[derive(Debug, Builder)]
pub struct Changeset {
    pub id: u32,
    pub created: TimestampFormat,
    #[builder(setter(strip_option), default)]
    pub closed: Option<TimestampFormat>,
    pub open: bool,
    #[builder(setter(strip_option), default)]
    pub uid: Option<i64>,
    #[builder(setter(strip_option), default)]
    pub user: Option<String>,
    pub tags: HashMap<String, String>,
    pub num_changes: u64,
    pub comments_count: u64,
}

impl Changeset {
    pub fn tag(&self, key: impl AsRef<str>) -> Option<&str> {
        self.tags.get(key.as_ref()).map(|s| s.as_str())
    }
    pub fn has_tag(&self, key: impl AsRef<str>) -> bool {
        self.tag(key).is_some()
    }
    pub fn num_tags(&self) -> usize {
        self.tags.len()
    }

    /// True iff this object has tags
    pub fn tagged(&self) -> bool {
        !self.untagged()
    }
    /// True iff this object has no tags
    pub fn untagged(&self) -> bool {
        self.num_tags() == 0
    }

    pub fn tags_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.tags
    }

    pub fn into_tags(self) -> HashMap<String, String> {
        self.tags
    }
}

pub struct ChangesetReader<R: Read> {
    reader: quick_xml::Reader<BufReader<R>>,
    buf: Vec<u8>,
}

impl<R: Read> ChangesetReader<R> {
    fn new(reader: R) -> ChangesetReader<R> {
        ChangesetReader {
            reader: quick_xml::Reader::from_reader(BufReader::new(reader)),
            buf: Vec::new(),
        }
    }

    fn next_changeset(&mut self) -> Result<Option<Changeset>> {
        // move forward until we are at a changeset tag (happens at the start)
        let changeset;
        loop {
            match self.reader.read_event(&mut self.buf)? {
                Event::Eof => {
                    return Ok(None);
                }
                Event::Start(ref e) => {
                    if e.name() != "changeset".as_bytes() {
                        continue;
                    }

                    let mut changeset_builder = ChangesetBuilder::default();
                    for attr in e.attributes() {
                        let attr = attr?;
                        match attr.key {
                            b"id" => {
                                changeset_builder
                                    .id(self.reader.decode(&attr.unescaped_value()?)?.parse()?);
                            }
                            b"created_at" => {
                                changeset_builder.created(TimestampFormat::ISOString(
                                    attr.unescape_and_decode_value(&self.reader)?,
                                ));
                            }
                            b"closed_at" => {
                                changeset_builder.closed(TimestampFormat::ISOString(
                                    attr.unescape_and_decode_value(&self.reader)?,
                                ));
                            }
                            b"open" => {
                                changeset_builder.open(match attr.value.as_ref() {
                                    b"true" => true,
                                    b"false" => false,
                                    _ => bail!("unknown value"),
                                });
                            }
                            b"user" => {
                                changeset_builder
                                    .user(attr.unescape_and_decode_value(&self.reader)?);
                            }
                            b"uid" => {
                                changeset_builder
                                    .uid(self.reader.decode(&attr.unescaped_value()?)?.parse()?);
                            }
                            b"num_changes" => {
                                changeset_builder.num_changes(
                                    self.reader.decode(&attr.unescaped_value()?)?.parse()?,
                                );
                            }
                            b"comments_count" => {
                                changeset_builder.comments_count(
                                    self.reader.decode(&attr.unescaped_value()?)?.parse()?,
                                );
                            }
                            _ => {}
                        }
                    }

                    // go for tags
                    let mut tags = HashMap::new();
                    let mut buf = Vec::new();
                    loop {
                        match self.reader.read_event(&mut buf)? {
                            Event::End(ref e) => {
                                if e.name() == "changeset".as_bytes() {
                                    break;
                                }
                            }
                            Event::Start(ref e) | Event::Empty(ref e) => {
                                if e.name() != "tag".as_bytes() {
                                    continue;
                                }
                                let mut k = None;
                                let mut v = None;
                                for attr in e.attributes() {
                                    let attr = attr?;
                                    match attr.key {
                                        b"k" => {
                                            k = Some(attr.unescape_and_decode_value(&self.reader)?);
                                        }
                                        b"v" => {
                                            v = Some(attr.unescape_and_decode_value(&self.reader)?);
                                        }
                                        _ => {}
                                    }
                                }
                                ensure!(k.is_some(), "No k for tag");
                                ensure!(v.is_some(), "No v for tag");
                                tags.insert(k.unwrap(), v.unwrap());
                            }
                            _ => continue,
                        }
                    }

                    changeset_builder.tags(tags);

                    changeset = Some(changeset_builder.build()?);
                    break;
                }
                Event::Empty(ref e) => {
                    if e.name() != "changeset".as_bytes() {
                        continue;
                    }

                    let mut changeset_builder = ChangesetBuilder::default();
                    for attr in e.attributes() {
                        let attr = attr?;
                        match attr.key {
                            b"id" => {
                                changeset_builder
                                    .id(self.reader.decode(&attr.unescaped_value()?)?.parse()?);
                            }
                            b"created_at" => {
                                changeset_builder.created(TimestampFormat::ISOString(
                                    attr.unescape_and_decode_value(&self.reader)?,
                                ));
                            }
                            b"closed_at" => {
                                changeset_builder.closed(TimestampFormat::ISOString(
                                    attr.unescape_and_decode_value(&self.reader)?,
                                ));
                            }
                            b"open" => {
                                changeset_builder.open(match attr.value.as_ref() {
                                    b"true" => true,
                                    b"false" => false,
                                    _ => bail!("unknown value"),
                                });
                            }
                            b"user" => {
                                changeset_builder
                                    .user(attr.unescape_and_decode_value(&self.reader)?);
                            }
                            b"uid" => {
                                changeset_builder
                                    .uid(self.reader.decode(&attr.unescaped_value()?)?.parse()?);
                            }
                            b"num_changes" => {
                                changeset_builder.num_changes(
                                    self.reader.decode(&attr.unescaped_value()?)?.parse()?,
                                );
                            }
                            b"comments_count" => {
                                changeset_builder.comments_count(
                                    self.reader.decode(&attr.unescaped_value()?)?.parse()?,
                                );
                            }
                            _ => {}
                        }
                    }

                    // no tags here
                    changeset_builder.tags(HashMap::new());

                    changeset = Some(changeset_builder.build()?);
                    break;
                }
                _ => continue,
            }
        }

        ensure!(changeset.is_some(), "No changeset created?!");
        Ok(Some(changeset.unwrap()))
    }
}

impl<R: Read> Iterator for ChangesetReader<R> {
    type Item = Result<Changeset>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_changeset().transpose()
    }
}

impl ChangesetReader<bzip2::read::MultiBzDecoder<std::fs::File>> {
    pub fn from_filename(filename: &str) -> Result<Self> {
        let f = File::open(filename)?;
        let dec = MultiBzDecoder::new(f);

        Ok(ChangesetReader::new(dec))
    }
}

pub fn changeset_file_to_tags(filename: &str) -> Result<Vec<Vec<(String, String)>>> {
    let osc = ChangesetReader::from_filename(filename)?;
    let mut results = Vec::new();
    let mut cid;
    let mut tags;
    let mut changeset;
    for changeset_res in osc.into_iter() {
        changeset = changeset_res?;
        cid = changeset.id as usize;
        tags = changeset.into_tags();
        if results.len() <= cid {
            results.resize(cid + 1, Vec::new());
        }
        results[cid] = tags.into_iter().collect();
    }

    Ok(results)
}
