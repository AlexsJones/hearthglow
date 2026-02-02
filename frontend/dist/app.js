// HearthGlow frontend (simple, DOM-based) — loads landing data and provides a create tab.

document.addEventListener("DOMContentLoaded", () => {
  const $ = (sel) => document.querySelector(sel);
  const tabLanding = $("#tab_landing");
  const tabStars = $("#tab_stars");
  const tabAdmin = $("#tab_admin");
  const landingSection = $("#landing");
  const starsSection = $("#stars");
  const adminSection = $("#admin");

  const createPersonBtn = $("#create_person");
  const createPersonResult = $("#create_person_result");
  const viewPersonBtn = $("#view_person");
  const personArea = $("#person_area");
  const personName = $("#person_name");
  const personDetails = $("#person_details");
  const chartsList = $("#charts_list");
  const createChartBtn = $("#create_chart");
  const chartResult = $("#chart_result");

  const calendarEl = $("#calendar");
  const eventPersonSelect = $("#event_person");
  const eventTitleInput = $("#event_title");
  const eventStartInput = $("#event_start");
  const eventEndInput = $("#event_end");
  const eventAddBtn = $("#event_add");
  const eventResult = $("#event_result");
  const eventFormPanel = $("#event_form_panel");
  const eventFormToggle = $("#event_form_toggle");
  const starsFocusList = $("#stars_focus_list");

  let currentPerson = null;
  let currentPersonId = null;
  let currentTab = "landing";
  let calendarInstance = null;
  const chartCooldownMs = 30000;
  const chartCooldowns = new Map();

  function canAddStar(chartId) {
    const last = chartCooldowns.get(chartId) || 0;
    return Date.now() - last >= chartCooldownMs;
  }

  function startCooldown(chartId) {
    chartCooldowns.set(chartId, Date.now());
  }

  function applyCooldownToButton(btn, chartId) {
    if (!btn) return;
    const last = chartCooldowns.get(chartId) || 0;
    const remaining = Math.max(0, chartCooldownMs - (Date.now() - last));
    if (remaining > 0) {
      btn.disabled = true;
      const seconds = Math.ceil(remaining / 1000);
      btn.textContent = `Wait ${seconds}s`;
      btn.dataset.cooldown = "1";
    } else {
      btn.disabled = false;
      btn.textContent = btn.dataset.defaultLabel || btn.textContent;
      btn.dataset.cooldown = "0";
    }
  }

  function scheduleCooldownTicker(btn, chartId) {
    if (!btn) return;
    if (btn.dataset.cooldownTicker) return;
    btn.dataset.cooldownTicker = "1";
    const tick = () => {
      applyCooldownToButton(btn, chartId);
      if (btn.disabled) {
        requestAnimationFrame(tick);
      } else {
        delete btn.dataset.cooldownTicker;
      }
    };
    tick();
  }

  function showTab(which) {
    currentTab = which;
    const isLanding = which === "landing";
    const isStars = which === "stars";
    tabLanding.classList.toggle("active", isLanding);
    tabStars.classList.toggle("active", isStars);
    tabAdmin.classList.toggle("active", which === "admin");
    landingSection.classList.toggle("hidden", !isLanding);
    starsSection.classList.toggle("hidden", !isStars);
    adminSection.classList.toggle("hidden", which !== "admin");
  }

  async function loadAdmin() {
    try {
      const peopleRes = await fetch("/admin/people");
      const people = peopleRes.ok ? await peopleRes.json() : [];
      const adminPeople = $("#admin_people_list");
      adminPeople.innerHTML = "";
      people.forEach((p) => {
        const li = document.createElement("li");
        const meta = document.createElement("div");
        meta.className = "chart-meta";
        meta.innerHTML = `<strong>${escapeHtml(p.first_name)} ${escapeHtml(p.last_name)}</strong><span class='small'>id: ${p.id}</span>`;
        const del = document.createElement("button");
        del.textContent = "Delete Person";
        del.addEventListener("click", async () => {
          if (!confirm(`Delete ${p.first_name}? This removes their charts.`))
            return;
          const r = await fetch(`/admin/people/${p.id}`, { method: "DELETE" });
          if (r.ok) {
            li.remove();
            await loadLanding();
            await loadAdmin();
          } else alert("Failed to delete person");
        });
        li.appendChild(meta);
        li.appendChild(del);
        adminPeople.appendChild(li);
      });

      // stars
      const starsRes = await fetch("/stars");
      const stars = starsRes.ok ? await starsRes.json() : [];
      const adminStars = $("#admin_stars_list");
      adminStars.innerHTML = "";
      stars.forEach((s) => {
        const li = document.createElement("li");
        const meta = document.createElement("div");
        meta.className = "chart-meta";
        meta.innerHTML = `<strong>${escapeHtml(s.name)}</strong><span class='small'>${escapeHtml(s.description)}</span>`;
        const controls = document.createElement("div");
        controls.className = "chart-controls";
        const countInput = document.createElement("input");
        countInput.type = "number";
        countInput.value = s.star_count;
        const totalInput = document.createElement("input");
        totalInput.type = "number";
        totalInput.value = s.star_total;
        const saveBtn = document.createElement("button");
        saveBtn.textContent = "Save";
        saveBtn.addEventListener("click", async () => {
          const payload = {
            name: s.name,
            description: s.description,
            star_count: parseInt(countInput.value),
            star_total: parseInt(totalInput.value),
          };
          const r = await fetch(`/stars/${s.id}`, {
            method: "PATCH",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(payload),
          });
          if (r.ok) {
            alert("Saved");
            await loadLanding();
            await loadAdmin();
          } else alert("Save failed");
        });
        const delBtn = document.createElement("button");
        delBtn.textContent = "Delete Chart";
        delBtn.addEventListener("click", async () => {
          if (!confirm("Delete this chart?")) return;
          const r = await fetch(`/admin/stars/${s.id}`, { method: "DELETE" });
          if (r.ok) {
            li.remove();
            await loadLanding();
            await loadAdmin();
          } else alert("Delete failed");
        });
        controls.appendChild(countInput);
        controls.appendChild(totalInput);
        controls.appendChild(saveBtn);
        controls.appendChild(delBtn);
        li.appendChild(meta);
        li.appendChild(controls);
        adminStars.appendChild(li);
      });
      // populate admin person select for creating charts
      const personSelect = $("#admin_chart_person_select");
      if (personSelect) {
        personSelect.innerHTML = "";
        people.forEach((p) => {
          const opt = document.createElement("option");
          opt.value = String(p.id);
          opt.textContent = `${p.first_name} ${p.last_name}`;
          personSelect.appendChild(opt);
        });
        // wire admin create button (replace existing handler if any)
        const adminCreateBtn = $("#admin_create_chart");
        if (adminCreateBtn) {
          adminCreateBtn.onclick = async () => {
            const selected = personSelect.value;
            const name = document
              .getElementById("admin_chart_name")
              .value.trim();
            const description = document
              .getElementById("admin_chart_description")
              .value.trim();
            const count =
              parseInt(document.getElementById("admin_chart_count").value) || 0;
            const total =
              parseInt(document.getElementById("admin_chart_total").value) ||
              10;
            if (!selected) {
              document.getElementById("admin_create_result").textContent =
                "Pick a person first";
              return;
            }
            try {
              const payload = {
                name,
                description,
                person_id: parseInt(selected),
                star_count: count,
                star_total: total,
              };
              const r = await fetch("/stars", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(payload),
              });
              if (!r.ok) throw new Error(await r.text());
              document.getElementById("admin_create_result").textContent =
                "Created chart";
              await loadAdmin();
              await loadLanding();
            } catch (err) {
              document.getElementById("admin_create_result").textContent =
                "Error: " + err.toString();
            }
          };
        }
      }
    } catch (err) {
      console.error("loadAdmin error", err);
    }
  }

  tabLanding.addEventListener("click", async () => {
    showTab("landing");
    await loadLanding();
  });
  tabStars.addEventListener("click", async () => {
    showTab("stars");
    await loadStarsFocus();
  });
  tabAdmin.addEventListener("click", async () => {
    showTab("admin");
    await loadAdmin();
  });

  // Load landing data: calendar + people
  async function loadLanding() {
    try {
      await ensureCalendar();
      await populateCalendarPeople();
    } catch (err) {
      console.error("loadLanding error", err);
    }
  }

  async function ensureCalendar() {
    if (!calendarEl || calendarInstance) return;
    if (!window.EventCalendar || !window.EventCalendar.create) {
      if (eventResult) {
        eventResult.textContent =
          "Calendar library failed to load. Please refresh.";
      }
      return;
    }

    // Fetch events and resources before creating calendar
    let events = [];
    let resources = [];
    try {
      const eventsRes = await fetch("/calendar/events");
      if (eventsRes.ok) {
        events = await eventsRes.json();
      }
      const resourcesRes = await fetch("/calendar/people");
      if (resourcesRes.ok) {
        resources = await resourcesRes.json();
      }
    } catch (err) {
      console.error("Failed to fetch calendar data:", err);
    }

    calendarInstance = window.EventCalendar.create(calendarEl, {
      view: "resourceTimeGridWeek",
      height: "auto",
      selectable: true,
      events: events,
      resources: resources,
      select: (info) => {
        // Only auto-fill form if the user clicked an empty slot, not when dragging events
        if (!info.event) {
          if (info.resource && eventPersonSelect) {
            eventPersonSelect.value = String(info.resource.id);
          }
          if (eventStartInput && eventEndInput) {
            eventStartInput.value = toLocalInputValue(info.start);
            eventEndInput.value = toLocalInputValue(info.end);
          }
        }
      },
      eventClick: (info) => {
        // Show event details in modal
        const event = info.event;
        const resource = event.resourceId ? resources.find(r => r.id === event.resourceId) : null;
        showEventModal(event, resource);
      },
      eventTimeFormat: { hour: "numeric", minute: "2-digit" },
      slotMinTime: "06:00:00",
      slotMaxTime: "21:00:00",
      nowIndicator: true,
    });
  }

  async function populateCalendarPeople() {
    if (!eventPersonSelect) return;
    const res = await fetch("/calendar/people");
    const people = res.ok ? await res.json() : [];
    eventPersonSelect.innerHTML = "";
    people.forEach((p) => {
      const opt = document.createElement("option");
      opt.value = String(p.id);
      opt.textContent = p.title || `${p.first_name || ""} ${p.last_name || ""}`;
      eventPersonSelect.appendChild(opt);
    });
  }

  function toLocalInputValue(dateObj) {
    const date = dateObj instanceof Date ? dateObj : new Date(dateObj);
    const pad = (v) => String(v).padStart(2, "0");
    return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(
      date.getDate()
    )}T${pad(date.getHours())}:${pad(date.getMinutes())}`;
  }

  function normalizeDateTime(value) {
    if (!value) return value;
    return value.length === 16 ? `${value}:00` : value;
  }

  function showEventModal(event, resource) {
    const modal = document.getElementById("event_modal");
    const title = document.getElementById("event_modal_title");
    const body = document.getElementById("event_modal_body");
    
    const startDate = new Date(event.start);
    const endDate = new Date(event.end);
    const personName = resource ? resource.title : "Unknown";
    
    const startStr = startDate.toLocaleString();
    const endStr = endDate.toLocaleString();
    
    title.textContent = event.title;
    body.innerHTML = `
      <p><strong>Person:</strong> ${escapeHtml(personName)}</p>
      <p><strong>Start:</strong> ${escapeHtml(startStr)}</p>
      <p><strong>End:</strong> ${escapeHtml(endStr)}</p>
    `;
    
    modal.classList.remove("hidden");
  }

  function closeEventModal() {
    const modal = document.getElementById("event_modal");
    modal.classList.add("hidden");
  }

  async function loadStarsFocus() {
    try {
      const sres = await fetch("/stars");
      const stars = sres.ok ? await sres.json() : [];
      starsFocusList.innerHTML = "";
      if (Array.isArray(stars) && stars.length) {
        stars.forEach((c) => {
          const li = document.createElement("li");
          li.className = "star-card";

          const header = document.createElement("div");
          header.className = "star-card-header";
          header.innerHTML = `
            <div class="star-title">${escapeHtml(c.name)}</div>
            <div class="star-owner">${escapeHtml(c.person_first_name)} ${escapeHtml(c.person_last_name)}</div>
          `;

          const desc = document.createElement("div");
          desc.className = "star-desc";
          desc.textContent = c.description || "Keep shining!";

          const meter = document.createElement("div");
          meter.className = "star-meter";
          meter.appendChild(createStarMeter(c.star_count, c.star_total));

          const count = document.createElement("div");
          count.className = "star-count";
          count.textContent = `${c.star_count} / ${c.star_total} stars`;

          if (c.star_count >= c.star_total) {
            li.classList.add("star-complete");
          }

          const btn = document.createElement("button");
          btn.className = "star-big-btn";
          btn.textContent = "Add a Star ⭐";
          btn.dataset.defaultLabel = "Add a Star ⭐";
          applyCooldownToButton(btn, c.id);
          if (btn.disabled) scheduleCooldownTicker(btn, c.id);
          btn.addEventListener("click", async (ev) => {
            if (!canAddStar(c.id)) {
              applyCooldownToButton(btn, c.id);
              scheduleCooldownTicker(btn, c.id);
              return;
            }
            try {
              await incrementChart(c.id, 1);
              startCooldown(c.id);
              applyCooldownToButton(btn, c.id);
              scheduleCooldownTicker(btn, c.id);
              showConfettiAtElement(ev.target);
            } catch (err) {
              console.error("increment failed", err);
            }
            await loadStarsFocus();
            await loadLanding();
          });

          li.appendChild(header);
          li.appendChild(desc);
          li.appendChild(meter);
          li.appendChild(count);
          li.appendChild(btn);
          starsFocusList.appendChild(li);
        });
      } else {
        starsFocusList.innerHTML =
          '<li class="small">No active charts yet</li>';
      }
    } catch (err) {
      console.error("loadStarsFocus error", err);
    }
  }

  if (eventAddBtn) {
    eventAddBtn.addEventListener("click", async () => {
      if (!eventPersonSelect || !eventTitleInput || !eventStartInput || !eventEndInput) {
        return;
      }
      const personId = parseInt(eventPersonSelect.value);
      const title = eventTitleInput.value.trim();
      const start = normalizeDateTime(eventStartInput.value);
      const end = normalizeDateTime(eventEndInput.value);

      if (!personId || !title || !start || !end) {
        if (eventResult) eventResult.textContent = "Please fill out all fields.";
        return;
      }

      if (eventResult) eventResult.textContent = "Saving...";
      try {
        const payload = {
          title,
          person_id: personId,
          start,
          end,
        };
        const res = await fetch("/calendar/events", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(payload),
        });
        if (!res.ok) throw new Error(await res.text());
        // Clear form after successful save
        eventTitleInput.value = "";
        eventStartInput.value = "";
        eventEndInput.value = "";
        if (eventResult) eventResult.textContent = "Event added!";
        // Refresh calendar to show new event
        if (calendarInstance) {
          setTimeout(async () => {
            try {
              const eventsRes = await fetch("/calendar/events");
              if (eventsRes.ok) {
                const events = await eventsRes.json();
                calendarInstance.setOption("events", events);
              }
            } catch (err) {
              console.error("Failed to refresh events:", err);
            }
          }, 100);
        }
      } catch (err) {
        if (eventResult) eventResult.textContent = `Error: ${err.toString()}`;
      }
    });
  }

  createPersonBtn.addEventListener("click", async () => {
    const first = document.getElementById("first_name").value.trim();
    const last = document.getElementById("last_name").value.trim();
    if (!first) return;
    createPersonResult.textContent = "Creating...";
    try {
      const res = await fetch("/people", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ first_name: first, last_name: last }),
      });
      if (!res.ok) throw new Error(await res.text());
      const body = await res.json();
      createPersonResult.textContent = `Created ${first} ${last} (id: ${body.id})`;
      currentPersonId = body.id;
      await loadLanding();
    } catch (err) {
      createPersonResult.textContent = "Error: " + err.toString();
    }
  });

  viewPersonBtn.addEventListener("click", async () => {
    const name = document.getElementById("view_name").value.trim();
    if (!name) return;
    await loadPerson(name);
  });

  createChartBtn.addEventListener("click", async () => {
    if (!currentPersonId) {
      chartResult.textContent =
        "Please create or view a person first (so we know the person id).";
      return;
    }
    const name = document.getElementById("chart_name").value.trim();
    const desc = document.getElementById("chart_description").value.trim();
    const count = parseInt(document.getElementById("chart_count").value) || 0;
    const total = parseInt(document.getElementById("chart_total").value) || 10;
    try {
      const res = await fetch("/stars", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          name,
          description: desc,
          person_id: currentPersonId,
          star_count: count,
          star_total: total,
        }),
      });
      if (!res.ok) throw new Error(await res.text());
      chartResult.textContent = "Chart created! Refreshing...";
      await loadPerson(currentPerson.first_name);
      await loadLanding();
    } catch (err) {
      chartResult.textContent = "Error: " + err.toString();
    }
  });

  async function loadPerson(firstName) {
    try {
      const res = await fetch(`/people/${encodeURIComponent(firstName)}`);
      if (res.status === 404) {
        alert("No such person");
        return;
      }
      if (!res.ok) throw new Error(await res.text());
      const body = await res.json();
      currentPerson = body;
      personName.textContent = `${body.first_name} ${body.last_name}`;
      currentPersonId = body.id;
      personDetails.innerHTML = "";
      chartsList.innerHTML = "";
      if (body.star_charts && body.star_charts.length) {
        body.star_charts.forEach((c) => {
          const li = document.createElement("li");
          const meta = document.createElement("div");
          meta.className = "chart-meta";
          meta.innerHTML = `<strong>${escapeHtml(c.name)}</strong><span class='small'>${escapeHtml(c.description)}</span><span class='small'>${c.star_count} / ${c.star_total}</span>`;
          if (c.star_count >= c.star_total) {
            li.classList.add("chart-complete");
            meta.innerHTML += `<span class='badge complete'>Complete</span>`;
          }
          const btn = document.createElement("button");
          btn.className = "increment";
          btn.textContent = "⭐ +1";
          btn.dataset.defaultLabel = "⭐ +1";
          applyCooldownToButton(btn, c.id);
          if (btn.disabled) scheduleCooldownTicker(btn, c.id);
          btn.addEventListener("click", async () => {
            if (!canAddStar(c.id)) {
              applyCooldownToButton(btn, c.id);
              scheduleCooldownTicker(btn, c.id);
              return;
            }
            await incrementChart(c.id, 1);
            startCooldown(c.id);
            applyCooldownToButton(btn, c.id);
            scheduleCooldownTicker(btn, c.id);
            await loadPerson(body.first_name);
            await loadLanding();
          });
          li.appendChild(meta);
          li.appendChild(btn);
          chartsList.appendChild(li);
        });
      } else {
        chartsList.innerHTML = '<li class="small">No charts yet</li>';
      }
      personArea.classList.remove("hidden");
      // remember id by creating via API or using create flow
    } catch (err) {
      alert("Error loading person: " + err.toString());
    }
  }

  async function incrementChart(id, delta) {
    const res = await fetch(`/stars/${id}/increment`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ delta }),
    });
    if (!res.ok) throw new Error(await res.text());
  }

  function showConfettiAtElement(el) {
    try {
      const rect = el.getBoundingClientRect();
      const colors = [
        "#ff8a65",
        "#ffd54f",
        "#ff7043",
        "#81c784",
        "#64b5f6",
        "#e57373",
      ];
      for (let i = 0; i < 18; i++) {
        const d = document.createElement("div");
        d.className = "confetti";
        d.style.background = colors[Math.floor(Math.random() * colors.length)];
        d.style.left =
          rect.left + rect.width / 2 + (Math.random() * 80 - 40) + "px";
        d.style.top =
          rect.top + window.scrollY + (Math.random() * 20 - 10) + "px";
        d.style.transform = `rotate(${Math.random() * 360}deg)`;
        document.body.appendChild(d);
        setTimeout(() => d.remove(), 1000);
      }
    } catch (e) {
      console.error("confetti", e);
    }
  }

  function createStarMeter(count, total) {
    const meter = document.createElement("div");
    meter.className = "star-meter-row";
    const maxDisplay = 12;
    const displayTotal = Math.max(1, Math.min(total || 0, maxDisplay));
    const displayCount = Math.min(count || 0, displayTotal);
    for (let i = 0; i < displayTotal; i++) {
      const s = document.createElement("span");
      s.className = "star-dot" + (i < displayCount ? " filled" : "");
      s.textContent = i < displayCount ? "★" : "☆";
      meter.appendChild(s);
    }
    return meter;
  }

  function escapeHtml(s) {
    return String(s)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");
  }

  // convenience: allow pressing Enter to view
  const viewName = document.getElementById("view_name");
  if (viewName)
    viewName.addEventListener("keypress", (e) => {
      if (e.key === "Enter") {
        document.getElementById("view_person").click();
      }
    });

  if (eventStartInput && eventEndInput) {
    const now = new Date();
    const start = new Date(now.getTime());
    start.setMinutes(0, 0, 0);
    start.setHours(start.getHours() + 1);
    const end = new Date(start.getTime() + 60 * 60 * 1000);
    eventStartInput.value = toLocalInputValue(start);
    eventEndInput.value = toLocalInputValue(end);
  }

  // Modal event listeners
  const eventModal = document.getElementById("event_modal");
  const eventModalClose = document.getElementById("event_modal_close");
  
  if (eventModalClose) {
    eventModalClose.addEventListener("click", closeEventModal);
  }
  
  if (eventModal) {
    eventModal.addEventListener("click", (e) => {
      // Close modal if clicking outside the modal content
      if (e.target === eventModal) {
        closeEventModal();
      }
    });
  }

  // Toggle event form panel
  if (eventFormToggle) {
    eventFormToggle.addEventListener("click", () => {
      if (eventFormPanel) {
        eventFormPanel.classList.toggle("hidden");
        eventFormToggle.textContent = eventFormPanel.classList.contains("hidden") 
          ? "+ Add Event" 
          : "- Close Form";
      }
    });
  }

  // initial landing load
  showTab("landing");
  loadLanding();

  console.log("HearthGlow frontend loaded");
});
