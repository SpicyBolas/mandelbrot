const canvas = document.getElementById('plot');

const WIDTH = canvas.width;
const HEIGHT = canvas.width;

const context = canvas.getContext('2d');
const MAX_ITER = 300;
let scaleFactor = 1;

//Define color array for the stability points
mBpalette = ['rgb(0,0,255)','rgb(32,107,203)','rgb(255,100,100)','rgb(255,170,100)','rgb(255,200,100)','rgb(0,255,0)'];

canvas.addEventListener('click',handleZoom);
canvas.addEventListener('contextmenu',handleZoomOut);

function getPoints() {
	const params = {
		height: HEIGHT,
		width: WIDTH,
		max_iter: MAX_ITER,
		scale_factor: scaleFactor
	};
	
	fetch('http://localhost:3000/post-mandelbrot-request', {
		method: "POST",
		body: JSON.stringify(params),
		headers: {
    		"Content-type": "application/json; charset=UTF-8"
  		}
	})
	.then((response) => response.json())
	.then((response) => {
		let points = response.points;
		//console.log(points);
		for (let i = 0; i < points.length; i++) {
			let color = `rgb(${points[i].color.red},${points[i].color.green},${points[i].color.blue})`;
			console.log(color);
			plotPoint(points[i].x, points[i].y, color, radius=1);
		}
		console.log('done');
	});
}

//Draw line between two points
function drawLine(src,dst) {
    context.beginPath();
    context.moveTo(src[0],src[1]);
    context.lineTo(dst[0],dst[1]);
    context.stroke();
}

//Point plotting function taking input pixel coordinates
function plotPoint(x,y,color,radius=5){
    context.beginPath();
    context.arc(x,y,radius,0,2 * Math.PI);
    context.fillStyle = color;
    context.fill();
	console.log('here with ' + color + ' x: ' + x + ', y: ' + y + ', rad: ' + radius);
}

function handleZoom(e){
    let xp = e.offsetX;
    let yp = e.offsetY;

    //convert the zoom coords to cartesian coords
    let [offsetX2,offsetY2] = pxToCart([xp,yp],scaleFactor,offsetX,offsetY);

    offsetX = offsetX2;
    offsetY = offsetY2;
    let SCALE_FACTOR2 = 1/10*scaleFactor;
    scaleFactor = SCALE_FACTOR2;

    for(let xp=0;xp<WIDTH+1;xp++){
        for(let yp=0;yp<HEIGHT+1;yp++){
            plotMandelbrot(xp,yp,100,SCALE_FACTOR2,offsetX,offsetY);
        }
    }
}

function handleZoomOut(e){
    e.preventDefault();
    let SCALE_FACTOR2 = Math.min(10*scaleFactor,1);
    scaleFactor = SCALE_FACTOR2;
    if(scaleFactor==1){
        offsetX = 0;
        offsetY = 0;
    } 

    for(let xp=0;xp<WIDTH+1;xp++){
        for(let yp=0;yp<HEIGHT+1;yp++){
            plotMandelbrot(xp,yp,100,SCALE_FACTOR2,offsetX,offsetY);
        }
    }
}


getPoints();
