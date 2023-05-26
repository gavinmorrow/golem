import ChatMessage, { messageForm } from "../custom-elements/ChatMessage.js";

function handleMessage(message) {
	console.log("Got message:", message);
	switch (Object.keys(message)[0]) {
		case "Authenticate":
			handleAuthenticate(message);
			break;
		case "NewMessage":
			handleNewMessage(message);
			break;
		case "Messages":
			handleMessages(message);
			break;
		case "Join":
			handleJoin(message);
			break;
		case "Error":
			handleError(message);
			break;
		default:
			console.error("Unknown message type:", message);
	}
}

function handleAuthenticate(message) {
	if (message.Authenticate.success) {
		console.log("Successfully authenticated!");

		// Delete the login button (already logged in)
		document.getElementById("show-login").remove();

		// Show send message form
		messageForm.style.display = "block";
	} else {
		console.error("Failed to authenticate:", message.Authenticate.error);
	}
}

function handleNewMessage(message) {
	console.log("New message:", message.NewMessage);

	message = message.NewMessage;

	const chat = document.getElementById("chat");
	makeMessageElem(message).insert();
}

function handleMessages(message) {
	console.log("Messages:", message.Messages);

	const messages = message.Messages;

	// Add to DOM

	for (const message of messages.reverse()) {
		makeMessageElem(message).insert();
	}
}

const joined = [];
const nickList = document.getElementById("nick-list");

function handleJoin(message) {
	console.log("Someone joined!", message.Join);
	joined.push(message.Join);

	// Add to nick list
	const name = message.Join.name;
	const elem = document.createElement("div");
	elem.textContent = name;
	nickList.appendChild(elem);
}

function makeMessageElem(message) {
	return new ChatMessage(message);
}

function handleError(message) {
	console.error("Error:", message.Error);
}

export default handleMessage;
