const COLOR_PALETTE = [
    [0.0 / 255.0, 0.0 / 255.0, 255.0 / 255.0],
	[32.0 / 255.0, 107.0 / 255.0, 203.0 / 255.0],
    [255.0 / 255.0, 100.0 / 255.0, 100.0 / 255.0],
    [255.0 / 255.0, 170.0 / 255.0, 100.0 / 255.0],
    [255.0 / 255.0, 200.0 / 255.0, 100.0 / 255.0],
    [0.0 / 255.0, 255.0 / 255.0, 0.0 / 255.0],
]; //divide by 255 to normalize into floating point for gpu
const SCALE_FACTOR = 1.0;
const OFFSET_X = 0.0;
const OFFSET_Y = 0.0;

const canvas = document.querySelector("canvas");

class Point {
	//expects x and y to be between -1.0 and 1.0
	//r, g and b should be between 0.0 and 1.0
	constructor(x, y, r, g, b) {
		this.x = x;
		this.y = y;
		this.r = r;
		this.g = g;
		this.b = b;
	}

	getAsArray() {
		return [this.x, this.y, this.r, this.g, this.b];
	}
}

class PointArray {
	//expects an array of Point objects or an empty array
	constructor(points = []) {
		this.points = points;
	}

	//returns a Float32Array [
	//    x, y, r, g, b
	// ]
	getAsFloat32Array() {
		let rawArray = []
		this.convertPointsToTriangle();
		this.points.forEach((point) => {
			rawArray.push(...point.getAsArray());
		});
		return new Float32Array(rawArray);
	}

	pushPoint(point) {
		this.points.push(point);
	}

	//TODO - make the size relate to a scale factor or similar
	convertPointsToTriangle() {
		let size = 0.1;
		let newPoints = [];
		this.points.forEach((point) => {
			//console.log(point);
			let po  = point.getAsArray();
			let x = po[0];
			let y = po[1];
			let r = po[2];
			let g = po[3];
			let b = po[4];
			newPoints.push(new Point(x, y, r, g, b));
			newPoints.push(new Point(x + size, y, r, g, b));
			newPoints.push(new Point(x, y + size, r, g, b));
			newPoints.push(new Point(x + size, y, r, g, b));
			newPoints.push(new Point(x, y + size, r, g, b));
			newPoints.push(new Point(x + size, y + size, r, g, b));
		});
		this.points = newPoints;
	}
}


async function initWebGPU() {

	if (!navigator.gpu) {
	  throw new Error("WebGPU not supported on this browser.");
	}

	const adapter = await navigator.gpu.requestAdapter();
	if (!adapter) {
		throw new Error("No GPUAdapter Found");
	}
	
	const device = await adapter.requestDevice();

	const context = canvas.getContext('webgpu'); 
	const canvasFormat = navigator.gpu.getPreferredCanvasFormat();
	context.configure({
		device: device,
		format: canvasFormat,
	});
	
	const vertices = calculateAllMandelbrotPoints(canvas.width, canvas.height, 100, 1);

	const encoder = device.createCommandEncoder();
	// start defining things for the GPU to do
	const pass = encoder.beginRenderPass({
		colorAttachments: [{
			view: context.getCurrentTexture().createView(),
			loadOp: "clear",
			clearValue: {r: 0.0, g: 0.0, b:0.0, a: 1}, //background color
			storeOp: "store",
		}]
	});

	// basically like preparing memory for the GPU
	const vertexBuffer = device.createBuffer({
		label: "verticies",
		size: vertices.byteLength,
		usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
	});

	//add our vertices to the vertexBuffer
	device.queue.writeBuffer(vertexBuffer, 0, vertices);

	const vertexBufferLayout = {
		arrayStride: 20, //number of bytes the GPU needs to skip to look at the next vertex (5 32bit floats per vertex, floats are 4bytes)
		attributes: [
		{
			format: "float32x2", //our vertices were defined at a 2D array of floats
			offset: 0,
			shaderLocation: 0, //parameter location
		},
		{
			format: "float32x3", //our color points
			offset: 8, //offset past the two earlier vertex points
			shaderLocation: 1,
		}
		],
	};

	// the code is the WGSL code
	// it is run once for every vertex
	const cellShaderModule = device.createShaderModule({
		label: "Cell shader",
		code: `
			struct VertexOutput {
			    @builtin(position) pos: vec4f,
			    @location(0) color: vec3f,
			};
			@vertex
			fn vertexMain(@location(0) pos: vec2f, @location(1) color: vec3f) -> VertexOutput {
				var out: VertexOutput;
				out.pos = vec4f(pos.x, pos.y, 0, 1);
				out.color = color;
				return out;
			}
			@fragment
			fn fragmentMain(@location(0) color: vec3f) -> @location(0) vec4f {
				return vec4f(color, 1.0); //1.0 is alpha value
			}
		`
	});
	const cellPipeline = device.createRenderPipeline({
		label: "Cell pipeline",
		layout: "auto",
		vertex: {
			module: cellShaderModule, 
			entryPoint: "vertexMain",
			buffers: [vertexBufferLayout]
		},
		fragment: {
			module: cellShaderModule,
			entryPoint: "fragmentMain",
			targets: [{
				format: canvasFormat //where to draw
			}]
		}
	});

	pass.setPipeline(cellPipeline);
	pass.setVertexBuffer(0, vertexBuffer);
	pass.draw(vertices.length / 5); //divide by 5 as each vertex contains (x, y, r, g, b) values
	pass.end();

	//actually submit the rendering to the GPU
	const commandBuffer = encoder.finish();
	device.queue.submit([commandBuffer]);
}

function calculateAllMandelbrotPoints(width, height, maxIterations) {
	let points = new PointArray();
	for (let i = 0; i < width; i++) {
		for (let j = 0; j < height; j++) {
			let point = calculateMandelbrotPointWithColor(i, j, maxIterations, height, width); 
			points.pushPoint(point)
		}
	}
	return points.getAsFloat32Array();
}

function calculateMandelbrotPointWithColor(xPos, yPos, maxIter, canvasHeight, canvasWidth) {
	let [xVertex, yVertex] = pixelToVertexCoordinates(xPos, yPos, canvasHeight, canvasWidth);
	let [x, y] = pixelToCartesianCoordinates(xPos, yPos, canvasHeight, canvasWidth);
	let c = [x, y];
	for (let i = 0; i < maxIter; i++) {
		let [xNew, yNew] = singleMandelbrotCalculation([x, y], c);
		let distance = Math.sqrt(xNew**2 + yNew**2);
		if (distance > 2) {
			let colorArr = COLOR_PALETTE[i % COLOR_PALETTE.length]; 
			return new Point(xVertex, yVertex, colorArr[0], colorArr[1], colorArr[2]);
		} 
		x = xNew;
		y = yNew;
	}
	return new Point(xVertex, yVertex, 0.0, 0.0, 0.0);
}

function singleMandelbrotCalculation(zPrev, c) {
	let a = zPrev[0];
	let b = zPrev[1];
	let newZTuple = [(a ** 2 - b ** 2) + c[0], 2.0 * a * b + c[1]];
	return newZTuple;
}

function pixelToVertexCoordinates(xPos, yPos, canvasHeight, canvasWidth) {
	let normalizedX = xPos / canvasWidth;
	let normalizedY = yPos / canvasHeight;
	let cartX = 2.0 * normalizedX - 1.0;
	let cartY = 2.0 * normalizedY - 1.0;
	return [cartX, cartY];
}

function pixelToCartesianCoordinates(xPos, yPos, canvasHeight, canvasWidth) {
	let axFactor = 2.0 * SCALE_FACTOR;
	let x = xPos / (canvasWidth / (2.0 * axFactor)) - axFactor + OFFSET_X; 
	let y = -yPos / (canvasHeight / (2.0 * axFactor)) + axFactor + OFFSET_Y;
	return [x, y];
}

function handleZoom(e){
    let xPos = e.offsetX;
    let yPos = e.offsetY;
    //convert the zoom coords to cartesian coords

    let [offsetX2,offsetY2] = pixelToCartesianCoordinates(xPos, yPos, canvas.height, canvas.width) 

    OFFSET_X = offsetX2;
    OFFSET_Y = offsetY2;
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
        OFFSET_X = 0;
        OFFSET_Y = 0;
    }


    for(let xp=0;xp<width+1;xp++){
        for(let yp=0;yp<height+1;yp++){
            plotMandelbrot(xp,yp,100,SCALE_FACTOR2,offsetX,offsetY);
        }
    }

}

initWebGPU();

canvas.addEventListener('click',handleZoom);
canvas.addEventListener('contextmenu',handleZoomOut);
