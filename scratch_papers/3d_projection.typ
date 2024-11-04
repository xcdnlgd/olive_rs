= 3D Projection
\

#import "@preview/cetz:0.3.0"

#align(center)[
#cetz.canvas({
    import cetz.draw: *

    let eye = (0, 0)
    let p0 = (2, 1)
    let p1 = (2, -1)
    let screen_x = 1

    set-style(
        mark: (fill: black),
        content: (padding: .2)
    )

    let marker(pos) = {
        circle(pos, fill: black, stroke: none, radius: .1)
    }

    line((-5,0), (5,0), mark: (end: "stealth"), name: "x_axis")
    content((), [$ x $], anchor: "west")
    line((0,-3), (0,3), mark: (end: "stealth"))
    content((), [$ y $], anchor: "south")

    marker(eye)
    content((), [$ "eye" $], anchor: "south-east")

    line((screen_x,-3), (screen_x,3), stroke: (dash: "dashed"), name: "screen")
    content((), [screen], anchor: "south")

    marker(p0)
    content((), [p0], anchor: "south-west")
    marker(p1)
    content((), [p2], anchor: "south-west")

    line(eye, p0, stroke: (dash: "dashed"), name: "l0")
    line(eye, p1, stroke: (dash: "dashed"), name: "l1")

    intersections("i", "screen", "l0", "l1")
    marker("i.0")
    content((), $ p'_0 $, anchor: "south")
    marker("i.1")
    content((), $ p'_1 $, anchor: "south")
})
]
