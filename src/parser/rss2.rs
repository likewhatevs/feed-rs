use super::{attr, text, timestamp, uuid_gen};
use chrono::prelude::*;
use entry::{Entry, Link};
use feed::Feed;
use xml5ever::rcdom::{Handle, NodeData};

static ATOM_NS: &'static str = "http://www.w3.org/2005/Atom";

pub fn handle_rss2(handle: Handle) -> Option<Feed> {
    let node = handle;
    let mut feed = Feed::new();
    handle_channel(node.clone(), &mut feed);
    for child in node.children.borrow().iter() {
        match child.data {
            NodeData::Element { ref name, .. } => {
                let tag_name = name.local.as_ref();
                match tag_name {
                    "channel" => handle_channel(child.clone(), &mut feed),
                    _ => (),
                }
            }
            _ => {}
        }
    }
    Some(feed)
}

pub fn handle_channel(handle: Handle, feed: &mut Feed) {
    let node = handle;
    for child in node.children.borrow().iter() {
        match child.data {
            NodeData::Element {
                ref name,
                ref attrs,
                ..
            } => {
                let tag_name = name.local.as_ref();
                let ns = name.ns.as_ref();
                match tag_name {
                    "title" => feed.title = text(child.clone()),
                    "description" => feed.description = text(child.clone()),
                    "link" => {
                        if ATOM_NS == ns {
                            let attributes = &attrs.borrow();
                            let href = attr("href", attributes);
                            let rel = attr("rel", attributes);
                            if let (Some(href), Some("self")) =
                                (href, rel.as_ref().map(String::as_ref))
                            {
                                feed.website = Some(href)
                            }
                        } else {
                            if let Some(url) = text(child.clone()) {
                                feed.website = Some(url)
                            }
                        }
                    }
                    "language" => feed.language = text(child.clone()),
                    "lastBuildDate" => feed.last_updated = timestamp(child.clone()),
                    "pubDate" => (),
                    "managingEditor" => (),
                    "webMaster" => (),
                    "copyright" => (),
                    "docs" => (),
                    "cloud" => (),
                    "ttl" => (),
                    "image" => feed.visual_url = image_url(child.clone()),
                    "textInput" => (),
                    "skipHours" => (),
                    "skipDays" => (),
                    "category" => {
                        //TODO
                    }
                    "item" => {
                        if let Some(entry) = handle_item(child.clone()) {
                            feed.entries.push(entry)
                        }
                    }
                    _ => (),
                }
            }
            _ => {}
        }
    }
}

pub fn image_url(handle: Handle) -> Option<String> {
    let node = handle;
    for child in node.children.borrow().iter() {
        match child.data {
            NodeData::Element { ref name, .. } => {
                let tag_name = name.local.as_ref();
                match tag_name {
                    "url" => return text(child.clone()),
                    _ => (),
                }
            }
            _ => (),
        }
    }
    None
}

pub fn handle_item(handle: Handle) -> Option<Entry> {
    let mut entry = Entry::new();
    let mut published: Option<DateTime<Utc>> = None;
    let node = handle;
    for child in node.children.borrow().iter() {
        match child.data {
            NodeData::Element {
                ref name,
                ref attrs,
                ..
            } => {
                let tag_name = name.local.as_ref();
                match tag_name {
                    "title" => entry.title = text(child.clone()),
                    "description" => entry.summary = text(child.clone()),
                    "link" => {
                        entry.alternate = text(child.clone())
                            .map(|s| vec![Link::new("text/html", s)])
                            .unwrap_or_else(|| vec![])
                    }
                    "author" => entry.author = text(child.clone()),
                    "category" => {
                        if let Some(s) = text(child.clone()) {
                            entry.keywords.push(s)
                        }
                    }
                    "comments" => {} // TODO
                    "enclosure" => {
                        let attributes = &attrs.borrow();
                        let mime_type = attr("type", attributes);
                        let length = attr("length", attributes).and_then(|s| s.parse::<i64>().ok());
                        let url = attr("url", attributes);
                        match (mime_type, length, url) {
                            (Some(mime_type), Some(length), Some(url)) => {
                                entry.enclosure.push(Link::enc(mime_type, length, url))
                            }
                            _ => (),
                        }
                    }
                    "guid" => entry.id = text(child.clone()).unwrap_or_else(uuid_gen),
                    "pubDate" => published = timestamp(child.clone()),
                    "source" => {} // TODO
                    _ => (),
                }
            }
            _ => (),
        }
    }
    entry.published = published.unwrap_or_else(Utc::now);
    Some(entry)
}
