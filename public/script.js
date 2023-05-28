import ws from "./js/ws.js";
import { messageForm } from "./js/custom-elements/ChatMessage.js";

let session = localStorage.getItem("session");
if (session != null) {
	// Attempt to login via session
	ws.send(
		JSON.stringify({
			AuthenticateToken: session,
		})
	);
}

document.getElementById("show-login").addEventListener("click", () => {
	document.getElementById("login-dialog").showModal();
});

document.getElementById("show-change-name").addEventListener("click", () => {
	document.getElementById("change-name-dialog").showModal();
});

document.getElementById("change-name-form").addEventListener("submit", event => {
	event.preventDefault();

	const name = document.getElementById("new-name").value;

	ws.send(
		JSON.stringify({
			ChangeName: name,
		})
	);

	document.getElementById("change-name-dialog").close();
});

messageForm.addEventListener("submit", event => {
	event.preventDefault();
	const content = messageForm.querySelector("#msg-input").value;
	const parent = messageForm.querySelector("#msg-input-parent").value;

	const msg = JSON.stringify({
		Message: {
			parent: parent.length <= 0 ? "0" : parent,
			content,
		},
	});

	console.log(msg);
	ws.send(msg);
});
messageForm.addEventListener("click", e => e.stopPropagation());

document.getElementById("login").addEventListener("submit", async event => {
	event.preventDefault();
	const name = document.getElementById("name").value;
	const password = document.getElementById("password").value;

	await login(name, password);

	// Also login this way, because otherwise we would have to reload the page.
	ws.send(
		JSON.stringify({
			Authenticate: {
				name,
				password,
			},
		})
	);

	document.getElementById("login-dialog").close();
});

async function login(name, password) {
	const res = await fetch("/api/login", {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify({
			name,
			password,
		}),
	});

	if (res.status === 404 /* Not Found */) {
		alert(`User ${name} not found!`);
		return;
	} else if (res.status === 401 /* Unauthorized */) {
		alert("Incorrect password!");
		return;
	} else if (res.status === 500 /* Internal Server Error */) {
		alert("There was an error.");
		return;
	}
}
