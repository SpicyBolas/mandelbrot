async function initWebGPU() {
    const canvas = document.querySelector("canvas");

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


	//TODO - loook at Index Buffers. basically define a point then have the GPU connect them
	const vertices = new Float32Array([
	//   X,    Y,
	  -0.8, -0.8,
	   0.8, -0.8,
	   0.8,  0.8,

	  -0.8,  -0.8,
	   0.8,  0.8,
		-0.8,  0.8,
	]);

	const encoder = device.createCommandEncoder();
	// start defining things for the GPU to do
	const pass = encoder.beginRenderPass({
		colorAttachments: [{
			view: context.getCurrentTexture().createView(),
			loadOp: "clear",
			clearValue: {r: 0.1, g: 0.5, b:0.7, a: 1},
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
		arrayStride: 8, //number of bytes the GPU needs to skip to look at the next vertex
		attributes: [{
			format: "float32x2", //our vertices were defined at a 2D array of floats
			offset: 0,
			shaderLocation: 0,
		}],
	};

	// the code is the WGSL code
	// it is run once for every vertex
	const cellShaderModule = device.createShaderModule({
		label: "Cell shader",
		code: `
			@vertex
			fn vertexMain(@location(0) pos: vec2f) -> @builtin(position) vec4f {
				return vec4f(pos.x, pos.y, 0, 1);
			}
			@fragment
			fn fragmentMain() -> @location(0) vec4f {
				return vec4f(1, 0, 0, 1);
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
	pass.draw(vertices.length / 2);

	pass.end();

	//actually submit the rendering to the GPU
	const commandBuffer = encoder.finish();
	device.queue.submit([commandBuffer]);
}

initWebGPU();
