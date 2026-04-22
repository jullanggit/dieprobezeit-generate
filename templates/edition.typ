#import "@preview/cades:0.3.1": qr-code

/* tunables */
#let editionNumber = EDITION;
#let date = datetime(year: YEAR, month: MONTH, day: DAY)
#let web = true

/* global modifications */
#set page(paper: "a4", margin: 6%)

#show link: it => if web { underline(text(fill: blue, it)) } else { it }

#set par(justify: true)

#let textSize = 13pt;
#set text(size: textSize)

#let headingSize = textSize * 2.2;
#show heading: set align(center)
#show heading: set par(justify: false)
#show heading: set text(weight: "regular")
#show heading.where(level: 1): set text(size: headingSize)
#show heading.where(level: 2): set text(size: (headingSize + textSize) / 2.2)

/* helpers */
#let balance(content) = layout(size => {
  let count = content.at("count")
  let textheight = measure(content, width: size.width).height / count
  let height = measure(content, height: textheight + 5pt, width: size.width).height
  block(height: height, content)
})
#let author(author) = [_verfasst von #{ author }_]

// TODO: make this auto-increment
#let superLink(str, num) = {
  super(link(str, [[#num]]))
};
#let reference(url, number, key) = if web {
  link(url)[#super(underline(text(fill: blue, [\[#number\]])))]
} else {
  cite(key)
}
#let httpLink(target) = link("https://" + target)[#target]
#let qr(content) = link(content)[#qr-code(content, error-correction: "Q")];
// langs
#let lang(lang, body) = {
  set text(lang: lang)
  body
}
#let de(body) = lang("de", body)
#let en(body) = lang("en", body)
// alignment
#let centered(body) = align(center)[#body]
#let columnar(body) = columns(2)[#body]
#let spacing = v(1fr)

/* logo */
#let colors = false
#let bigSize = 114pt;
#let logo = {
  grid(
    columns: 3,
    rows: (24pt, auto, 24pt),
    grid.cell(
      fill: if colors { green },
      x: 0,
      y: 0,
      rowspan: 2,
      align(left, stack(
        dir: ltr,
        align(bottom + center)[
          #context {
            let spacingElemRation = 0.3
            let stack = stack(spacing: spacingElemRation * 1em, [D], [I], [E])
            text(
              size: measure(text(size: bigSize, [P])).height / measure(stack).height * 1em,
              stack,
            )
          }
        ],
        text(size: bigSize, [P$r o b epsilon$]),
      )),
    ),
    grid.cell(
      fill: if colors { red },
      x: 2,
      y: 0,
      align(right, {
        let germanWeekdays = (
          "Montag",
          "Dienstag",
          "Mittwoch",
          "Donnerstag",
          "Freitag",
          "Samstag",
          "Sonntag",
        )
        let weekday = germanWeekdays.at(date.weekday() - 1)
        text(size: bigSize / 7.3, [#weekday, #date.display("[day].[month].[year]")])
      }),
    ),
    grid.cell(
      fill: if colors { blue },
      x: 1,
      y: 1,
      colspan: 2,
      rowspan: 2,
      align(right + bottom, text(size: bigSize, $ZZ epsilon i tau$)),
    ),
    grid.cell(
      fill: if colors { purple },
      y: 2,
      {
        v(bigSize / 19)
        block(below: 5pt, line(length: 100%, stroke: 2pt))
        text(size: bigSize / 6.33, [Ausgabe #editionNumber])
      },
    ),
  )
  block(above: bigSize / 19, line(length: 100%, stroke: 2pt))
}

#context {
  page(
    stack(
      dir: ttb,
      logo,
      spacing,
      de(centered[
        Wir informieren alle 1-2 Monate mehrsprachig über aktuelle Geschehnisse in der Schule, der
        Schweiz und der Welt, fassen interessante Forschungsarbeiten in verschiedenen Disziplin zusammen und publizieren Kurzgeschichten, Artikel und andere Einsendungen aus der
        Schülerschaft. \
        Diese sind neben dieser Papierversion auch online unter #httpLink("dieprobezeit.ch") verfügbar,
        einen QR-Code dazu findet ihr auf der Hinterseite. \
        Hinter der Probe Zeit steht ein kleines Redaktionsteam sowie eine Gruppe Journalist:innen,
        der jeder und jede beitreten kann, also falls du auch etwas schreiben willst, kontaktiere uns
        doch über #link("mailto:team@dieprobezeit.ch")[team\@dieprobezeit.ch] \
        Form von Events, wissenschaftlichen Themen und einer spannenden Kurzgeschichte zu finden. \
        #text(size: textSize * 0.9, [_Layout, Design und Vorwort von Aster_])
      ]),
      spacing,
      balance(columnar([
        PREVIEWS
      ])),
    ),
    footer: de({
      block(below: 6pt, line(length: 100%))
      columnar({
        set text(size: 10pt)
        grid(
          gutter: 1%,
          columns: (auto, 1fr),
          image("Brainmade.svg", height: 20pt),
          [
            _Es befinden sich keine KI-generierten
            Inhalte in dieser Zeitung_
          ],
        )
        colbreak()
        [
          _Wir berichten unabhängig und haben keinerlei
          Verbindungen zu jeglichen Quellen_ // TODO: formulate this a bit nicer
        ]
      })
    }),
  )
}

#pagebreak()

BODY

#de(centered(stack(
  spacing,
  box(
    width: 80%,
    stroke: 1pt,
    inset: (y: 8%, rest: 5%),
    [
      = Vielen Dank fürs Lesen!
      Diese, sowie alte und zukünftige Ausgaben sind ebenfalls unter #httpLink("dieprobezeit.ch") zu finden
      = Deine Meinung zählt!
      Die Probezeit ist eine Zeitung von Schüler:innen für
      Schüler:innen! Würdest du ein Thema gerne in der nächsten Ausgabe sehen?
      Willst du einen Artikel oder eine Kurzgeschichte schreiben oder dich anderweitig
      beteiligen? \
      Dann kontakiere uns doch gerne über
      #httpLink("dieprobezeit.ch/feedback") oder #link("mailto:team@dieprobezeit.ch")[team\@dieprobezeit.ch]
      \ \
      Wir freuen uns auf dich in der nächsten Ausgabe!
    ],
  ),
  if not web {
    v(10%)
    qr("https://dieprobezeit.ch")
    v(15pt)
    [Scanne den QR-Code um die Onlineversion zu lesen!]
  },
  spacing,
)))


#if web {
  show bibliography: it => none
}

#bibliography(
  "refs.yaml",
  style: "link-references.csl",
  title: if web { none } else { [Quellen] },
)
