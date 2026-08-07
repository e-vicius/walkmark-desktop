<script lang="ts">
	import { sensitivityLabel } from "$lib/format";
	import { store } from "$lib/store.svelte";
	import { Label } from "$lib/components/ui/label";
	import { Slider } from "$lib/components/ui/slider";
	import { Switch } from "$lib/components/ui/switch";

	const settings = $derived(store.settings);

	function patchCapture(next: Partial<typeof settings.capture>) {
		void store.updateSettings({ capture: { ...settings.capture, ...next } });
	}
</script>

<div class="space-y-5">
	<p class="text-sm text-(--text)/56">
		How Steppy watches your screen and turns changes into steps. These apply to every new
		recording.
	</p>

	<div class="grid gap-5 sm:grid-cols-2">
		<div class="flex flex-col gap-2">
			<div class="flex items-center justify-between">
				<Label>How eagerly to capture</Label>
				<span class="text-xs text-(--text)/56">{sensitivityLabel(settings.capture.sensitivity)}</span>
			</div>
			<Slider
				type="single"
				value={settings.capture.sensitivity}
				min={0}
				max={1}
				step={0.05}
				onValueChange={(v: number) => patchCapture({ sensitivity: v })}
			/>
		</div>

		<div class="flex flex-col gap-2">
			<div class="flex items-center justify-between">
				<Label>Minimum gap between steps</Label>
				<span class="text-xs text-(--text)/56">{(settings.capture.minGapMs / 1000).toFixed(1)}s</span>
			</div>
			<Slider
				type="single"
				value={settings.capture.minGapMs}
				min={0}
				max={6000}
				step={100}
				onValueChange={(v: number) => patchCapture({ minGapMs: v })}
			/>
		</div>

		<div class="flex flex-col gap-2">
			<div class="flex items-center justify-between">
				<Label>Countdown before starting</Label>
				<span class="text-xs text-(--text)/56">
					{settings.capture.countdownSecs === 0 ? "None" : `${settings.capture.countdownSecs}s`}
				</span>
			</div>
			<Slider
				type="single"
				value={settings.capture.countdownSecs}
				min={0}
				max={10}
				step={1}
				onValueChange={(v: number) => patchCapture({ countdownSecs: v })}
			/>
		</div>

		<div class="flex flex-col gap-2">
			<div class="flex items-center justify-between">
				<Label>How often to check the screen</Label>
				<span class="text-xs text-(--text)/56">
					{(settings.capture.sampleIntervalMs / 1000).toFixed(1)}s
				</span>
			</div>
			<Slider
				type="single"
				value={settings.capture.sampleIntervalMs}
				min={200}
				max={3000}
				step={100}
				onValueChange={(v: number) => patchCapture({ sampleIntervalMs: v })}
			/>
		</div>

		<div class="flex flex-col gap-2 sm:col-span-2">
			<div class="flex items-center justify-between">
				<Label>Stored screenshot width</Label>
				<span class="text-xs text-(--text)/56">{settings.capture.maxWidth}px</span>
			</div>
			<Slider
				type="single"
				value={settings.capture.maxWidth}
				min={1000}
				max={3000}
				step={100}
				onValueChange={(v: number) => patchCapture({ maxWidth: v })}
			/>
			<p class="text-xs text-(--text)/56">
				Frames are downscaled to this width when captured. Lower saves disk space; higher keeps
				fine UI text sharp.
			</p>
		</div>
	</div>

	<div class="space-y-4 border-t border-(--text)/8 pt-4">
		<div class="flex items-center justify-between gap-4">
			<div>
				<Label>Wait for the screen to settle</Label>
				<p class="text-xs text-(--text)/56">
					Avoids capturing half-open menus and mid-flight animations.
				</p>
			</div>
			<Switch
				checked={settings.capture.settle}
				onCheckedChange={(settle) => patchCapture({ settle })}
			/>
		</div>
		<div class="flex items-center justify-between gap-4">
			<div>
				<Label>Get out of the way while recording</Label>
				<p class="text-xs text-(--text)/56">
					Minimises Steppy and shows a small floating controller instead.
				</p>
			</div>
			<Switch
				checked={settings.capture.hideWindow}
				onCheckedChange={(hideWindow) => patchCapture({ hideWindow })}
			/>
		</div>
	</div>
</div>
