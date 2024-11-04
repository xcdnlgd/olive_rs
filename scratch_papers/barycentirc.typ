#import "@preview/cetz:0.3.0"

#align(center)[
#cetz.canvas({
    import cetz.draw: *
    // Your drawing code goes here
    let (a, b, c, p) = ((0,0), (1,2), (3,-1), (1,0.5))
    line(a, b, c, close: true, stroke: gray)
    line(a, p, stroke: (paint: gray, dash: "dashed"))
    line(b, p, stroke: (paint: gray, dash: "dashed"))
    line(c, p, stroke: (paint: gray, dash: "dashed"))
    circle(a, radius: .05, fill: black, stroke: none)
    content(a, [v0], anchor: "west", padding: 2pt)
    circle(b, radius: .05, fill: black, stroke: none)
    content(b, [v1], anchor: "west", padding: 2pt)
    circle(c, radius: .05, fill: red, name: "p2")
    content(c, [v2], anchor: "west", padding: 2pt)
    circle(c, radius: .05, fill: black, stroke: none)
    content(p, [p], anchor: "west", padding: 2pt)
    circle(p, radius: .05, fill: black, stroke: none)
})
]
