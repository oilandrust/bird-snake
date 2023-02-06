# BirdSnake

## Play the game on the browser using webassebmly.

[Try it now!](https://oliver-rust.itch.io/bird-snake)

## A Snakebird clone writen in rust using the Bevy engine.

Snakebird is a famous puzzle game derived from snake known for it's funky movement mechanics. It is also known to have very hard puzzles depite it's cute and friendly graphics.

## Scope

I implemented the basic mechanics and reproduced the first 20 levels of the game. Some of the mechanics involved in the later levels such as teleports have not been implemented.

The main features are:

* Snake-like movement on a grid with smooth transitions.
* Gravity and pushing things around.
* Undo mechanics.
* Rendering the snake as a vector graphic.

## Why making a Snakebird clone?

This project was intended for fun and learning rust and the bevy engine. I am not a game designer so copying a game allowed me to not worry about the game being fun and have a something to implement without suffering from the "writer block".
I also enjoy this game and I was curious to implement smooth movement on a grid, the state of the game is fixed on a grid but the movement are smooth.

## Interesting systems.

The most fun and chalenging systems were the movement system with undo, including gravity, collision detection and allowing pushing things. Making that work with undo was quite interesting. The solution relies on a hashmap of the gid positions that are occupied by entity as well as commands that encapsulate movement of the entities as well as update of the hasmap.

Another interesing part was rendering the snake with vector graphics. The snake always occupies a list of grid positions but during transitions, parts of the snake stretch and become not square which was challenging to get right.