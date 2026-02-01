// HearthGlow frontend (simple, DOM-based) — loads landing data and provides a create tab.

document.addEventListener('DOMContentLoaded', () => {
  const $ = (sel) => document.querySelector(sel);
  const tabLanding = $('#tab_landing');
  const tabAdmin = $('#tab_admin');
  const landingSection = $('#landing');
  const adminSection = $('#admin');

  const createPersonBtn = $('#create_person');
  const createPersonResult = $('#create_person_result');
  const viewPersonBtn = $('#view_person');
  const personArea = $('#person_area');
  const personName = $('#person_name');
  const personDetails = $('#person_details');
  const chartsList = $('#charts_list');
  const createChartBtn = $('#create_chart');
  const chartResult = $('#chart_result');

  const landingCharts = $('#landing_charts');
  const familyList = $('#family_list');

  let currentPerson = null;
  let currentPersonId = null;

  function showTab(which) {
    if (which === 'landing') {
      tabLanding.classList.add('active');
      tabAdmin.classList.remove('active');
      landingSection.classList.remove('hidden');
      adminSection.classList.add('hidden');
    } else {
      tabLanding.classList.remove('active');
      tabAdmin.classList.add('active');
      landingSection.classList.add('hidden');
      adminSection.classList.remove('hidden');
    }
  }

  async function loadAdmin() {
    try {
      const peopleRes = await fetch('/admin/people');
      const people = peopleRes.ok ? await peopleRes.json() : [];
      const adminPeople = $('#admin_people_list');
      adminPeople.innerHTML = '';
      people.forEach(p => {
        const li = document.createElement('li');
        const meta = document.createElement('div');
        meta.className = 'chart-meta';
        meta.innerHTML = `<strong>${escapeHtml(p.first_name)} ${escapeHtml(p.last_name)}</strong><span class='small'>id: ${p.id}</span>`;
        const del = document.createElement('button');
        del.textContent = 'Delete Person';
        del.addEventListener('click', async () => {
          if (!confirm(`Delete ${p.first_name}? This removes their charts.`)) return;
          const r = await fetch(`/admin/people/${p.id}`, { method: 'DELETE' });
          if (r.ok) { li.remove(); await loadLanding(); await loadAdmin(); }
          else alert('Failed to delete person');
        });
        li.appendChild(meta);
        li.appendChild(del);
        adminPeople.appendChild(li);
      });

      // stars
      const starsRes = await fetch('/stars');
      const stars = starsRes.ok ? await starsRes.json() : [];
      const adminStars = $('#admin_stars_list');
      adminStars.innerHTML = '';
      stars.forEach(s => {
        const li = document.createElement('li');
        const meta = document.createElement('div');
        meta.className = 'chart-meta';
        meta.innerHTML = `<strong>${escapeHtml(s.name)}</strong><span class='small'>${escapeHtml(s.description)}</span>`;
        const controls = document.createElement('div');
        controls.className = 'chart-controls';
        const countInput = document.createElement('input');
        countInput.type = 'number'; countInput.value = s.star_count;
        const totalInput = document.createElement('input');
        totalInput.type = 'number'; totalInput.value = s.star_total;
        const saveBtn = document.createElement('button'); saveBtn.textContent = 'Save';
        saveBtn.addEventListener('click', async () => {
          const payload = { name: s.name, description: s.description, star_count: parseInt(countInput.value), star_total: parseInt(totalInput.value) };
          const r = await fetch(`/stars/${s.id}`, { method: 'PATCH', headers: {'Content-Type':'application/json'}, body: JSON.stringify(payload) });
          if (r.ok) { alert('Saved'); await loadLanding(); await loadAdmin(); } else alert('Save failed');
        });
        const delBtn = document.createElement('button'); delBtn.textContent = 'Delete Chart';
        delBtn.addEventListener('click', async () => {
          if (!confirm('Delete this chart?')) return;
          const r = await fetch(`/admin/stars/${s.id}`, { method: 'DELETE' });
          if (r.ok) { li.remove(); await loadLanding(); await loadAdmin(); } else alert('Delete failed');
        });
        controls.appendChild(countInput); controls.appendChild(totalInput); controls.appendChild(saveBtn); controls.appendChild(delBtn);
        li.appendChild(meta); li.appendChild(controls);
        adminStars.appendChild(li);
      });
      // populate admin person select for creating charts
      const personSelect = $('#admin_chart_person_select');
      if (personSelect) {
        personSelect.innerHTML = '';
        people.forEach(p => {
          const opt = document.createElement('option');
          opt.value = String(p.id);
          opt.textContent = `${p.first_name} ${p.last_name}`;
          personSelect.appendChild(opt);
        });
        // wire admin create button (replace existing handler if any)
        const adminCreateBtn = $('#admin_create_chart');
        if (adminCreateBtn) {
          adminCreateBtn.onclick = async () => {
            const selected = personSelect.value;
            const name = document.getElementById('admin_chart_name').value.trim();
            const description = document.getElementById('admin_chart_description').value.trim();
            const count = parseInt(document.getElementById('admin_chart_count').value) || 0;
            const total = parseInt(document.getElementById('admin_chart_total').value) || 10;
            if (!selected) { document.getElementById('admin_create_result').textContent = 'Pick a person first'; return; }
            try {
              const payload = { name, description, person_id: parseInt(selected), star_count: count, star_total: total };
              const r = await fetch('/stars', { method: 'POST', headers: {'Content-Type':'application/json'}, body: JSON.stringify(payload) });
              if (!r.ok) throw new Error(await r.text());
              document.getElementById('admin_create_result').textContent = 'Created chart';
              await loadAdmin();
              await loadLanding();
            } catch (err) {
              document.getElementById('admin_create_result').textContent = 'Error: ' + err.toString();
            }
          };
        }
      }
    } catch (err) { console.error('loadAdmin error', err); }
  }

  tabLanding.addEventListener('click', () => showTab('landing'));
  tabAdmin.addEventListener('click', async () => { showTab('admin'); await loadAdmin(); });

  // Load landing data: /stars and /people
  async function loadLanding() {
    try {
      // stars
      const sres = await fetch('/stars');
      const stars = sres.ok ? await sres.json() : [];
      landingCharts.innerHTML = '';
      if (Array.isArray(stars) && stars.length) {
        stars.forEach(c => {
          const li = document.createElement('li');
          const meta = document.createElement('div');
          meta.className = 'chart-meta';
          meta.innerHTML = `<strong>${escapeHtml(c.name)}</strong><span class='small'>${escapeHtml(c.description)}</span><span class='small'>${c.star_count} / ${c.star_total}</span><span class='small'>Owner: ${escapeHtml(c.person_first_name)} ${escapeHtml(c.person_last_name)}</span>`;
          // mark complete charts visually
          if (c.star_count >= c.star_total) {
            li.classList.add('chart-complete');
            meta.innerHTML += `<span class='badge complete'>Complete</span>`;
          }
          const btn = document.createElement('button');
          btn.className = 'increment';
          btn.textContent = '⭐ +1';
          btn.addEventListener('click', async (ev) => {
            try { await incrementChart(c.id, 1, ev.target); showConfettiAtElement(ev.target); } catch (err) { console.error('increment failed', err); }
            await loadLanding();
          });
          li.appendChild(meta);
          li.appendChild(btn);
          landingCharts.appendChild(li);
        });
      } else {
        landingCharts.innerHTML = '<li class="small">No active charts</li>';
      }

      // family
      const pres = await fetch('/people');
      const family = pres.ok ? await pres.json() : [];
      familyList.innerHTML = '';
      family.forEach(name => {
        const el = document.createElement('li');
        el.className = 'family-pill';
        const initial = (name && name.length) ? name.trim().charAt(0).toUpperCase() : '?';
        el.innerHTML = `
          <div class="family-avatar">${escapeHtml(initial)}</div>
          <div class="family-info">
            <div class="family-name">${escapeHtml(name)}</div>
          </div>
        `;
        familyList.appendChild(el);
      });
    } catch (err) {
      console.error('loadLanding error', err);
    }
  }

  createPersonBtn.addEventListener('click', async () => {
    const first = document.getElementById('first_name').value.trim();
    const last = document.getElementById('last_name').value.trim();
    if (!first) return;
    createPersonResult.textContent = 'Creating...';
    try {
      const res = await fetch('/people', {
        method: 'POST', headers: {'Content-Type':'application/json'}, body: JSON.stringify({ first_name: first, last_name: last })
      });
      if (!res.ok) throw new Error(await res.text());
      const body = await res.json();
      createPersonResult.textContent = `Created ${first} ${last} (id: ${body.id})`;
      currentPersonId = body.id;
      await loadLanding();
    } catch (err) {
      createPersonResult.textContent = 'Error: ' + err.toString();
    }
  });

  viewPersonBtn.addEventListener('click', async () => {
    const name = document.getElementById('view_name').value.trim();
    if (!name) return;
    await loadPerson(name);
  });

  createChartBtn.addEventListener('click', async () => {
    if (!currentPersonId) {
      chartResult.textContent = 'Please create or view a person first (so we know the person id).';
      return;
    }
    const name = document.getElementById('chart_name').value.trim();
    const desc = document.getElementById('chart_description').value.trim();
    const count = parseInt(document.getElementById('chart_count').value) || 0;
    const total = parseInt(document.getElementById('chart_total').value) || 10;
    try {
      const res = await fetch('/stars', {
        method: 'POST', headers: {'Content-Type':'application/json'}, body: JSON.stringify({ name, description: desc, person_id: currentPersonId, star_count: count, star_total: total })
      });
      if (!res.ok) throw new Error(await res.text());
      chartResult.textContent = 'Chart created! Refreshing...';
      await loadPerson(currentPerson.first_name);
      await loadLanding();
    } catch (err) {
      chartResult.textContent = 'Error: ' + err.toString();
    }
  });

  async function loadPerson(firstName) {
    try {
      const res = await fetch(`/people/${encodeURIComponent(firstName)}`);
      if (res.status === 404) { alert('No such person'); return; }
      if (!res.ok) throw new Error(await res.text());
      const body = await res.json();
      currentPerson = body;
      personName.textContent = `${body.first_name} ${body.last_name}`;
      currentPersonId = body.id;
      personDetails.innerHTML = '';
      chartsList.innerHTML = '';
      if (body.star_charts && body.star_charts.length) {
        body.star_charts.forEach(c => {
          const li = document.createElement('li');
          const meta = document.createElement('div');
          meta.className = 'chart-meta';
          meta.innerHTML = `<strong>${escapeHtml(c.name)}</strong><span class='small'>${escapeHtml(c.description)}</span><span class='small'>${c.star_count} / ${c.star_total}</span>`;
          if (c.star_count >= c.star_total) {
            li.classList.add('chart-complete');
            meta.innerHTML += `<span class='badge complete'>Complete</span>`;
          }
          const btn = document.createElement('button');
          btn.className = 'increment';
          btn.textContent = '⭐ +1';
          btn.addEventListener('click', async () => { await incrementChart(c.id, 1); await loadPerson(body.first_name); await loadLanding(); });
          li.appendChild(meta);
          li.appendChild(btn);
          chartsList.appendChild(li);
        });
      } else {
        chartsList.innerHTML = '<li class="small">No charts yet</li>';
      }
      personArea.classList.remove('hidden');
      // remember id by creating via API or using create flow
    } catch (err) { alert('Error loading person: ' + err.toString()); }
  }

  async function incrementChart(id, delta) {
    const res = await fetch(`/stars/${id}/increment`, { method: 'POST', headers: {'Content-Type':'application/json'}, body: JSON.stringify({ delta }) });
    if (!res.ok) throw new Error(await res.text());
  }

  function showConfettiAtElement(el) {
    try {
      const rect = el.getBoundingClientRect();
      const colors = ['#ff8a65','#ffd54f','#ff7043','#81c784','#64b5f6','#e57373'];
      for (let i=0;i<18;i++) {
        const d = document.createElement('div');
        d.className = 'confetti';
        d.style.background = colors[Math.floor(Math.random()*colors.length)];
        d.style.left = (rect.left + rect.width/2 + (Math.random()*80-40)) + 'px';
        d.style.top = (rect.top + window.scrollY + (Math.random()*20-10)) + 'px';
        d.style.transform = `rotate(${Math.random()*360}deg)`;
        document.body.appendChild(d);
        setTimeout(()=> d.remove(), 1000);
      }
    } catch(e){ console.error('confetti', e); }
  }

  function escapeHtml(s){ return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;'); }

  // convenience: allow pressing Enter to view
  const viewName = document.getElementById('view_name');
  if (viewName) viewName.addEventListener('keypress', (e)=>{ if(e.key==='Enter'){ document.getElementById('view_person').click(); }});

  // initial landing load
  showTab('landing');
  loadLanding();

  console.log('HearthGlow frontend loaded');
});
