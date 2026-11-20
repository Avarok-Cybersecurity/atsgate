// The ATS-safe resume template: one column, standard headings, real text, no tables, the contact
// details in the body. All text comes from the data file as strings, so nothing in it is ever
// interpreted as markup. Spacing is part of the contract with atsgate-resume: lines inside one
// paragraph sit closer together than separate paragraphs, which is how wrapped lines are rejoined.
#let data = json("/resume.json")

#set document(title: data.name, author: data.name)
#set page(paper: "us-letter", margin: (x: 0.75in, y: 0.75in), numbering: none)
#set text(font: "Libertinus Serif", size: 10pt, ligatures: false, hyphenate: false, lang: "en")
#set par(justify: false, leading: 0.45em, spacing: 0.75em)

// Typst may break a line after a hyphen or a slash, and a parser rejoining the lines cannot tell
// "smart-contract" from "smart- contract". Words holding either are kept whole on one line.
#let whole(s) = s.split(" ").map(w => if w.contains("-") or w.contains("/") { box(w) } else { w }).join(" ")

#let block-of(b) = {
  if b.kind == "name" {
    par(text(size: 18pt, weight: "bold", b.text))
  } else if b.kind == "title" {
    par(text(size: 11.5pt, b.text))
  } else if b.kind == "contact" {
    par(whole(b.text))
  } else if b.kind == "heading" {
    v(0.5em)
    block(sticky: true, par(text(size: 12pt, weight: "bold", b.text)))
  } else if b.kind == "paragraph" {
    par(whole(b.text))
  } else if b.kind == "entry" {
    v(0.25em)
    block(sticky: true, par[#text(weight: "bold", whole(b.title))#whole(b.rest)])
  } else if b.kind == "bullet" {
    pad(left: 0.6em, par(hanging-indent: 0.8em, [#data.bullet#whole(b.text)]))
  } else if b.kind == "labeled" {
    par(hanging-indent: 1.2em)[#text(weight: "bold", b.label + ":") #whole(b.text)]
  } else {
    panic("unknown block kind: " + b.kind)
  }
}

#for b in data.blocks { block-of(b) }
