/**
 * Counts down the "resend code" wait; the API allows a new code once a minute.
 * Call `stop` when the component is destroyed.
 */
export function createResendCountdown(seconds = 60) {
	let remaining = $state(0);
	let timer: ReturnType<typeof setInterval> | undefined;

	function stop() {
		clearInterval(timer);
	}

	function start() {
		stop();
		remaining = seconds;
		timer = setInterval(() => {
			remaining -= 1;
			if (remaining <= 0) stop();
		}, 1000);
	}

	return {
		get remaining() {
			return remaining;
		},
		start,
		stop
	};
}
