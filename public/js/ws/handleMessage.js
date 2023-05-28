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
		case "Leave":
			handleLeave(message);
			break;
		case "Update":
			handleUpdate(message);
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
	elem.setAttribute("data-presence-id", message.Join.id);
	nickList.appendChild(elem);
}

function handleLeave(message) {
	console.log("Someone left!", message.Leave);

	// Remove from nick list
	const id = message.Leave.id;
	const elem = nickList.querySelector(`[data-presence-id="${id}"]`);
	elem.remove();
}

function handleUpdate(message) {
	console.log("Update:", message.Update);

	// Update nick list
	const id = message.Update.id;
	const elem = nickList.querySelector(`[data-presence-id="${id}"]`);
	elem.textContent = message.Update.name;
}

function makeMessageElem(message) {
	return new ChatMessage(message);
}

function handleError(message) {
	console.error("Error:", message.Error);
}

export default handleMessage;
