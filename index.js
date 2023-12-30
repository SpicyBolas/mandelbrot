const canvas = document.getElementById('plot');

const width = canvas.width;
const height = canvas.width;

const context = canvas.getContext('2d');
const MAX_ITER = 300;
const SCALE_FACTOR = 1;

//Define color array for the stability points
mBpalette = ['rgb(0,0,255)','rgb(32,107,203)','rgb(255,100,100)','rgb(255,170,100)','rgb(255,200,100)','rgb(0,255,0)'];

canvas.addEventListener('click',handleZoom);
canvas.addEventListener('contextmenu',handleZoomOut);

const webSocket = new WebSocket('ws://localhost:3000/ws');

webSocket.onopen = (event) => {
	//information we send to the backend 
	const params = {
		height: height,
		width: width,
		max_iter: MAX_ITER,
		scale_factor: SCALE_FACTOR
	};
	webSocket.send(JSON.stringify(params));
};

webSocket.onmessage = (event) => {

};

for(let xp=0;xp<width+1;xp++){
    for(let yp=0;yp<height+1;yp++){
        plotMandelbrot(xp,yp,100);
    }
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
}

function handleZoom(e){
    let xp = e.offsetX;
    let yp = e.offsetY;

	let pixelArray = 

    //convert the zoom coords to cartesian coords
    let [offsetX2,offsetY2] = pxToCart([xp,yp],SCALE_FACTOR,offsetX,offsetY);

    offsetX = offsetX2;
    offsetY = offsetY2;
    let SCALE_FACTOR2 = 1/10*SCALE_FACTOR;
    SCALE_FACTOR = SCALE_FACTOR2;

    for(let xp=0;xp<width+1;xp++){
        for(let yp=0;yp<height+1;yp++){
            plotMandelbrot(xp,yp,100,SCALE_FACTOR2,offsetX,offsetY);
        }
    }
}

function handleZoomOut(e){
    e.preventDefault();
    let SCALE_FACTOR2 = Math.min(10*SCALE_FACTOR,1);
    SCALE_FACTOR = SCALE_FACTOR2;
    if(SCALE_FACTOR==1){
        offsetX = 0;
        offsetY = 0;
    } 

    for(let xp=0;xp<width+1;xp++){
        for(let yp=0;yp<height+1;yp++){
            plotMandelbrot(xp,yp,100,SCALE_FACTOR2,offsetX,offsetY);
        }
    }
}




