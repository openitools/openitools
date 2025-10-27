<script lang="ts">
	import { onMount } from 'svelte';

	// you don't have to touch any of those
	//
	// those are gotten from the `gst-plugin-rs` project, it's an api for their WebRTC
	import GstWebRTCAPI from './gst-webrtc-api';
	import type ComChannel from './gst-webrtc-api/com-channel';

	// the video element it self
	//
	// it's in a variable so we can have more control to it, like pushing the video stream
	let videoEl: HTMLVideoElement;

	// you don't need to wory about this, it sets up the config
	const signalingProtocol = window.location.protocol.startsWith('https') ? 'wss' : 'ws';
	const gstWebRTCConfig = {
		meta: { name: `WebClient-${Date.now()}` },
		signalingServerUrl: `${signalingProtocol}://${window.location.hostname}:8443`
	};

	const api = new GstWebRTCAPI(gstWebRTCConfig);

	function producerAddedFunc(producerId) {
		console.log(`called producer add`, producerId);
		const session = api.createConsumerSession(producerId.id);
		session.mungeStereoHack = true;
		console.log(`got the session`, session);
		// session.mungeStereoHack = true;
		session.addEventListener('error', (event) => {
			console.error(event.message, event.error);
		});

		// this runs once the device disconnects the screen mirroring, so we need to remove the video element
		session.addEventListener('closed', () => {
			videoEl.pause();
			videoEl.srcObject = null;
		});

		// this runs once there's a new stream, or a new frames of video coming, so we change the stream to the new one
		session.addEventListener('streamsChanged', () => {
			console.log('stream changed');
			const streams = session.streams;
			console.log(streams);
			if (streams.length > 0) {
				videoEl.srcObject = streams[0];

				// not sure why, it is what it is
				videoEl.play().catch((r) => {
					console.log(`error playing the video: ${r}`);
				});
			}
		});

		// IDK :)
		session.addEventListener('remoteControllerChanged', () => {
			console.log('remote controller changed');
			const remoteController = session.remoteController;
			console.log(remoteController);
			if (remoteController) {
				remoteController.attachVideoElement(videoEl);
				remoteController.addEventListener('info', (e) => {
					console.log('Received info message from producer: ', e.detail);
				});
			}
		});

		// and finally starts the stuff
		session.connect();
	}

	onMount(() => {
		// // check if webrtc work in the webview
		// console.log(typeof RTCPeerConnection);

		// adds a listener for which when a new producer, or a stream, or a device
		api.registerPeerListener({
			producerAdded: producerAddedFunc
		});

		// loops through all the available devices, and runs them
		//
		// probably not needed, but that's what I saw in the other project that does this
		for (const producer of api.getAvailableProducers()) {
			console.log(`new producer: ${producer}`);
			producerAddedFunc(producer);
		}
	});
</script>

<!-- `bind:this` binds the element it self to a variable -->
<video bind:this={videoEl} autoplay playsinline muted> </video>
