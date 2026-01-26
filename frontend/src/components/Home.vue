<script setup>
</script>

<template>
    <div class="classrooms-page">
    <!-- Header with search and subject filter only -->
    <header class="page-header">
        <h1>Classrooms</h1>
        <div class="header-actions">
        <div class="search-container">
            <input v-model="searchQuery" placeholder="Search classrooms..." class="search-input" />
        </div>
        <select v-model="selectedSubject" class="subject-select">
            <option value="">All Subject</option>
            <option v-for="disc in subject" :key="disc" :value="disc">{{ disc }}</option>
        </select>
        </div>
    </header>

    <!-- Extended grid: classrooms + add card -->
    <div class="classrooms-grid">
        <!-- Filtered classroom cards -->
        <div
        v-for="classroom in filteredClassrooms"
        :key="classroom.id"
        class="classroom-card">

        <div class="card-header">
            <h3>{{ classroom.name }}</h3>
            <div class="card-actions">
            <button @click="editClassroom(classroom)">Edit</button>
            <button @click="deleteClassroom(classroom.id)" class="delete-btn">Delete</button>
            </div>
        </div>
        <div class="card-body">
            <p><strong>Subject:</strong> {{ classroom.subject }}</p>
            <p><strong>Students:</strong> {{ classroom.students }} / {{ classroom.capacity }}</p>
            <p><strong>Teacher:</strong> {{ classroom.teacher }}</p>
        </div>
        <div class="card-footer">
            <router-link :to="`/classroom/${classroom.id}`" class="enter-btn">Enter Classroom</router-link>
        </div>
        </div>

        <!-- Add Classroom Card - positioned beside grid -->
        <div class="classroom-card add-card" @click="openAddModal">
        <div class="add-content">
            <div class="plus-icon">➕</div>
            <h3>Add Classroom</h3>
            <p>Create new classroom</p>
        </div>
        </div>
    </div>

    <!-- Add Modal -->
    <div v-if="showAddModal" class="modal-overlay" @click="closeAddModal">
        <div class="modal" @click.stop>
        <h2>Add New Classroom</h2>
        <form @submit.prevent="addClassroom">
            <input v-model="newClassroom.name" placeholder="Classroom Name" required />
            <select v-model="newClassroom.subject" required>
            <option value="">Select Subject</option>
            <option v-for="disc in subject" :key="disc" :value="disc">{{ disc }}</option>
            </select>
            <input v-model.number="newClassroom.capacity" type="number" placeholder="Capacity" required />
            <input v-model="newClassroom.teacher" placeholder="Teacher Name" required />
            <div class="modal-actions">
            <button type="submit">Create</button>
            <button type="button" @click="closeAddModal">Cancel</button>
            </div>
        </form>
        </div>
    </div>
    </div>
</template>

<style>
</style>