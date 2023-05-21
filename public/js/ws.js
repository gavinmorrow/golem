import handleMessage from "./ws/handleMessage.js";

// Connect to the websocket endpoint
const HOST = "localhost:7878";
const ws = new WebSocket(`ws://${HOST}/api/ws/`);

ws.addEventListener("open", event => {
	console.log("Connected to the websocket endpoint!");

	// Load messages
	getMessages(ws);
});

ws.addEventListener("message", event => {
	const data = JSON.parse(event.data);
	if (data != null) handleMessage(JSON.parse(event.data));
	else console.error("Invalid message received:", event.data);
});

ws.addEventListener("error", event => {
	console.log("Error", event);
});

ws.addEventListener("close", event => {
	console.log("Disconnected from the websocket endpoint!");
});

function getMessages(ws) {
	ws.send(
		JSON.stringify({
			LoadAllMessages: null,
		})
	);
}

export default ws;
