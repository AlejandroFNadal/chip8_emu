//Draw a circle in canvas chip8
var canvas = document.getElementById("chip8");
var ctx = canvas.getContext("2d");
ctx.moveTo(0, 0);
ctx.lineTo(100, 100);
ctx.stroke();
