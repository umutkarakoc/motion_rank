let player;
let drawing = null;
let draw;
let line;
let video_width = 0;
let video_heigth = 0;
let is_ready = false;
let drawings = [];
let time = 0;
let color = null;


function init_popovers() {
	const popoverTriggerList = document.querySelectorAll('[data-bs-toggle="popover"]')
	popoverTriggerList.forEach(popoverTriggerEl => new bootstrap.Popover(popoverTriggerEl))
	const tooltipTriggerList = document.querySelectorAll('[data-bs-toggle="tooltip"]')
	tooltipTriggerList.forEach(tooltipTriggerEl => new bootstrap.Popover(tooltipTriggerEl, {trigger: "hover"}))
}
$(init_popovers)

$(function() {
	player = videojs(document.getElementById("player"), {
		liveui: true,
		inactivityTimeout: 999999999,
		controlBar: {
			volumePanel: {
				inline: false
			}
		}
	});

	// $(".vjs-control-bar")
	// 	.appendTo("#controls")

	draw = SVG(document.getElementById("canvas")).size("100%", "100%");

	let viewbox = $("#canvas").attr("viewBox").split(" ");
	video_width = Number(viewbox[2]);
	video_heigth = Number(viewbox[3]);


	let render_time = (time) => {
		time = time / 1000
		let minute = Math.floor(time / 60);
		let second = Math.floor(time % 60);
		if (second < 10)
			second = "0" + second;
		if (minute < 10)
			minute = "0" + minute;

		return { minute, second }
	}


	async function update_time() {
		if (!is_ready) return;

		time = Math.floor(await player.currentTime() * 1000);

		let t = render_time(time)
		// let review_end_time = render_time( Math.min(time + duration, Math.floor(video_duration)) );

		if (document.getElementById("timer"))
			document.getElementById("timer").innerText = `${t.minute}:${t.second}`
	}

	// update_time();
	player.on("timeupdate", update_time);
	player.on("ready", function() {
		is_ready = true;
		first_interact()
	})
	player.on("play", clean_drawings)

	let selected_time = $(".review.selected").attr("data-time");
	set_player_time(Number(selected_time || 0));

})

function select_color(e) {
	$(".color-pick").css("border-width", "1px");
	$("#canvas").css("cursor", "crosshair")

	if (e.target.id == "color_000000") {
		$(e.target).css("border", "3px dotted #ffffff")
	}
	else {
		$(e.target).css("border-width", "3px")
	}

	color = e.target.id.replace("color_", "#");
}
function clean_drawings() {
	drawings = []
	$("polyline").remove();
	$(".color-pick").css("border-width", "1px");
	color = null;
}

let first_interacted = false;

function first_interact(e) {
	if (
		e &&
		(
			$(e.target).hasClass("vjs-icon-placeholder") ||
			$(e.target).hasClass("vjs-tech")
		)
	) {
		first_interacted = true
		return;
	}
	if (!is_ready)
		return;
	if (!first_interacted) {
		setTimeout(() => {
			player.play().then(() => {
				first_interacted = true;
				player.pause();
			})
		}, 100)
	}
}


function canvas_mousedown(e) {
	if (color == null){
		if(player.paused()){
			player.play()
		} else {
			player.pause()
		}
		return;
	}

	if (!first_interacted)
		return

	player.pause();
	drawing = [];

	let width_factor = video_width / $("#canvas").innerWidth();
	let height_factor = video_heigth / $("#canvas").innerHeight();
	let line_width = video_width * 0.01;

	let x = Math.floor(e.offsetX * width_factor);
	let y = Math.floor(e.offsetY * height_factor);

	drawing.push(x);
	drawing.push(y);

	line = draw.polyline(drawing)
		.stroke({ color, width: line_width, linecap: 'round' })
		.fill("none")
}



function canvas_mousemove(e) {
	if (drawing == null) return;

	let width_factor = video_width / $("#canvas").innerWidth();
	let height_factor = video_heigth / $("#canvas").innerHeight();

	let x = Math.floor(e.offsetX * width_factor);
	let y = Math.floor(e.offsetY * height_factor);

	drawing.push(x);
	drawing.push(y);

	line.plot(drawing)// .map((a,i) => i % 2 == 0 ? a * 0.01 * width : a * 0.01 * height ));

}

async function canvas_mouseup() {
	if (drawing == null) return;
	drawings.push({
		drawing,
		color
	});
	line.clear();
	drawing = null;
	line = null;
}

async function set_player_time(time_to) {
	let time = Math.floor(await player.currentTime() * 1000);

	if (time != time_to) {
		player.currentTime(time_to / 1000);
	}
	player.play().then(() => {
		player.pause();
	})
	clean_drawings();
}

document.onkeydown = async () => {

};

function write_review_click() {
	player.pause();
}


let on_request = false;
async function submit() {
	if (on_request)
		return;

	try {
		on_request = true;
		let text = $("#write_review_txt").val();
		let review = {
			drawings,
			text,
			time,
			percent: (time / 1000) / player.duration() * 100
		}

		console.log(review)

		let response = await fetch(location.pathname + "review/", {
			method: "POST",
			body: JSON.stringify(review),
			headers: {
				"content-type": "application/json"
			}
		});

		let result = await response.json();

		await htmx.ajax("GET", location.pathname+ "review/?selected=" + result.id  ,
			{
				target: "#reviews",
				swap: "innerHTML"
			}
		);

		$("#write_review_txt").val("")
		clean_drawings()
		player.pause();
	}
	catch (e) {

	}
	finally {
		on_request = false;
	}
}


$("body").click(first_interact)
$(first_interact)

function check_review_collapse () {
	// let reviews = $(".review");
	// reviews.each(function(i) {
	// 	let x = $(this).position().left;

	// 	reviews.each(function(j) {
	// 		if(i == j) 
	// 			return;

	// 		let ox = $(this).position().left;
	// 		if(ox < x  15)
			
	// 	})
	// })
}
