var canvas = document.getElementById('plot');

const width = canvas.width;
const height = canvas.width;
//canvas.addEventListener('onclick');

const context = canvas.getContext('2d');

const center = width/2; //height and width are the same so just one value is fine


var SCALE_FACTOR = 1;
var offsetX = 0;
var offsetY = 0;

//Convert cartesian to pixel coordinates from array input.
//Take the scale factor and offsets as input
function cartToPx(p,SF=1,xOffset,yOffset) {
    let axFactor = 2*SF;

    var xp = width/(2*axFactor)*((p[0]-xOffset)+axFactor);
    var yp = -height/(2*axFactor)*((p[1]-yOffset)-axFactor);

    return [xp,yp];
}

//Convert Pixel to cartesian
function pxToCart(p_p,SF=1,xOffset,yOffset) {
    let axFactor = 2*SF;

    var x = p_p[0]/(width/(2*axFactor))-axFactor+xOffset;
    var y = -p_p[1]/(height/(2*axFactor))+axFactor+yOffset;

    return [x,y];
}

//Define color array for the stability points
mBpalette = ['rgb(0,0,255)','rgb(32,107,203)','rgb(255,100,100)','rgb(255,170,100)','rgb(255,200,100)','rgb(0,255,0)'];




//Get the first 5 iterations of mandelbrot

function mandelbrotIter(c,iter) {
    let z_arr = [[...c]];
    for (i=0;i<iter;i++){
        //real part
        let a = z_arr[z_arr.length-1][0];
        //imaginary part
        let b = z_arr[z_arr.length-1][1];
        //perform the iteration
        z_arr2 = [...z_arr,[(a**2-b**2)+c[0],2*a*b+c[1]]];
        z_arr = [...z_arr2];
    }
    return z_arr;
}

function mandelbrot(z_prev,c) {
    //real part
    let a = z_prev[0];
    //imaginary part
    let b = z_prev[1];
    //perform the iteration
    z_arr2 = [(a**2-b**2)+c[0],2*a*b+c[1]];
    return z_arr2;
}


//Draw line between two points
function drawLine(p1,p2) {
    context.beginPath();
    context.moveTo(p1[0],p1[1]);
    context.lineTo(p2[0],p2[1]);
    context.stroke();
}

//Point plotting function taking input pixel coordinates
function plotPoint(x,y,color,radius=5){
    context.beginPath();
    context.arc(x,y,radius,0,2 * Math.PI);
    context.fillStyle = color;
    context.fill();
}


//Plot the mandelbrot set
    
//function for plotting
function plotMandelbrot(xp,yp,MAX_ITER,SF=1,offsetX=0,offsetY=0){
    //convert to cartesian
    let [x,y] = pxToCart([xp,yp],SF,offsetX,offsetY);
    let c = [x,y];
    //Iterate using the logistic equation
    for(let iter=0;iter<MAX_ITER;iter++){
        let [x_new,y_new] = mandelbrot([x,y],c);
        
        let distance = Math.sqrt(x_new**2+y_new**2);
        if(distance >2){
            let hue = Math.floor(100*iter/MAX_ITER); 

            //let color_in = 'hsl(' + hue.toString() + '%,100%,100%)' 
            let color_in = mBpalette[iter % mBpalette.length];
            plotPoint(xp,yp,color=color_in,radius=1);
            return;
        }
        x = x_new;
        y = y_new;
    }
    plotPoint(xp,yp,color='black',radius=1);
}

for(let xp=0;xp<width+1;xp++){
    for(let yp=0;yp<height+1;yp++){
        plotMandelbrot(xp,yp,100);
    }
}



//Create event listener to plot point when clicked/mousemove
/*function handleEvent(e){
    context.clearRect(0,0,width,height)
    let xp = e.offsetX
    let yp = e.offsetY

    //convert to cartesian
    let [x,y] = pxToCart([xp,yp]);
    //Iterate using the logistic equation
    let z_arr = mandelbrotIter([x,y],iter=10);
    //Convert back to pixel coordinates
    let z_arr_px = z_arr.map(cartToPx);
    console.log(z_arr_px);
    for(let i=0;i<z_arr_px.length;i++){
        plotPoint(z_arr_px[i][0],z_arr_px[i][1],color=palette[i % palette.length])
        if(i>0){
            drawLine(z_arr_px[i],z_arr_px[i-1])
        }
    }

}

canvas.addEventListener('mousemove',handleEvent)*/
function handleEvent(e){
    
    let xp = e.offsetX
    let yp = e.offsetY

    //convert to cartesian
    let [x,y] = pxToCart([xp,yp]);
    var coords = document.getElementById('coords');
    coords.innerHTML = 'Real: ' + x.toString() + ', Imaginary: ' + y.toString();


}

function handleZoom(e){
    let xp = e.offsetX;
    let yp = e.offsetY;
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


canvas.addEventListener('click',handleZoom);
canvas.addEventListener('contextmenu',handleZoomOut);


