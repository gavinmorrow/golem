import ws from "./js/ws.js";
import { messageForm } from "./js/custom-elements/ChatMessage.js";

document.getElementById("show-login").addEventListener("click", () => {
	document.getElementById("login-dialog").showModal();
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

document.getElementById("login").addEventListener("submit", event => {
	event.preventDefault();
	const name = document.getElementById("name").value;
	const password = document.getElementById("password").value;

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
