let player;
let drawing = null;
let draw;
let line;
let video_width = 0;
let video_heigth = 0;
let is_ready = false;


$(function(){
	player = videojs( document.getElementById("player"), {
		liveui: true,
		inactivityTimeout: 999999999
	});

	$(".vjs-control-bar")
		.appendTo("#controls")

	draw = SVG(document.getElementById("canvas")).size("100%", "100%");

	let viewbox = $("#canvas").attr("viewBox").split(" ");
	video_width = Number(viewbox[2]);
	video_heigth = Number(viewbox[3]);

	$(".vjs-progress-control").on("click", async function() {

	})
	// setTimeout(() => {
	// 	$(".vjs-control-bar").css({ "background" : "none"})
	// }, 500)

	let render_time = (time) => {
		time = time / 1000
		let minute = Math.floor(time / 60);
		let second = Math.floor(time % 60);
		if (second < 10)
			second = "0" + second;
		if (minute < 10)
			minute = "0" + minute;

		return {minute, second}
	}

	let path = location.pathname.split("/");
	// let time = Number(path[4]);
	// let video_id = path[3];
	// player.currentTime( time )

	async function update_time (e) {
		if(!is_ready) return;

		let time = Math.floor( await player.currentTime() * 1000 );

		if(e.manuallyTriggered) {
			await htmx.ajax("GET", location.href.split("?")[0], 
				{
					swap: "multi:#reviews,#canvas",
					"push-url": "true"
				}
			);
		}
		// let time = await player.currentTime();
		// let video_duration = player.duration() || 0;
		// // let duration = 3;

		let review_time = render_time(time)
		// let review_end_time = render_time( Math.min(time + duration, Math.floor(video_duration)) );

		if(document.getElementById("time"))
			document.getElementById("time").value =  time;
		// // document.getElementById("duration").value = duration;
		if(document.getElementById("timer"))
			document.getElementById("timer").innerText = `${review_time.minute}:${review_time.second}`
	}

	// update_time();
	player.on("timeupdate", update_time);
	player.on("ready", function() { is_ready = true;})

	let selected_time = $(".review.selected").attr("data-time");
	set_player_time(Number(selected_time || 0));

})

let color = null ;
function select_color (e) {
	$("#canvas").show();

	$(".color-pick").css("border", "none");
	$("#canvas").css("cursor", "crosshair")

	if(e.target.id == "color_000000") {
		$(e.target).css("border", "3px dotted #ffffff")
	}
	else {
		$(e.target).css("border", "3px dotted #000")
	}

	color = e.target.id.replace("color_", "#");
}

let first_interacted = false;

function first_interact (e) {
	if(!first_interacted) {
		$("#canvas").show();
		first_interacted = true;
		player.play().then(() => player.pause());
	}
}


function canvas_mousedown(e){
	if (color == null) return;

	player.pause();
	drawing = [];

	let width_factor = video_width / $("#canvas").innerWidth();
	let height_factor = video_heigth / $("#canvas").innerHeight() ;
	let line_width = video_width * 0.01;

	let x = Math.floor(e.offsetX * width_factor) ;
	let y = Math.floor(e.offsetY * height_factor) ;

	drawing.push(x);
	drawing.push(y);

	line = draw.polyline(drawing)
		.stroke({ color, width: line_width , linecap: 'round' })
		.fill("none")
}

function canvas_mousemove(e) {
	if(drawing == null) return;

	let width_factor = video_width / $("#canvas").innerWidth();
	let height_factor = video_heigth / $("#canvas").innerHeight() ;

	let x = Math.floor(e.offsetX * width_factor) ;
	let y = Math.floor(e.offsetY * height_factor) ;

	drawing.push(x);
	drawing.push(y);

	line.plot(drawing)// .map((a,i) => i % 2 == 0 ? a * 0.01 * width : a * 0.01 * height ));

}

async function canvas_mouseup(e) {
	if(drawing == null) return;
	let _drawing = drawing;
	line.clear();
	drawing = null;
	line = null;

	let time = Math.floor(await player.currentTime() * 1000);

	let parts = location.search.replace("?","").split("&");
	let review_id = null;

	for (var i = 0; i < parts.length; i++) {
		if (parts[i].startsWith("review=")){
			review_id = parts[i].replace("review=", "");
			break;;
		}
	}

	if(review_id == null)
		return;

	let {url} =  await fetch(location.pathname + "drawing", {
		method: "POST",
		body: JSON.stringify({ review_id, color, drawing: _drawing }),
		headers : {
			"content-type": "application/json"
		}
	});

	await htmx.ajax("GET", location.href, 
		{
			swap: "multi:#side,#canvas",
		}
	);


}

async function set_player_time(time_to) {
	console.log("side", time_to)
	let time = Math.floor(await player.currentTime() * 1000);

	if(time != time_to) {
		player.currentTime(time_to / 1000);
	}
}

document.onkeydown = async (e) => {

};

function write_review_click(e){
	player.pause();
}