import { beforeEach, describe, expect, it } from 'vitest';
import { TestBed } from '@angular/core/testing';

import { sampleWithNewData, sampleWithRequiredData } from '../job-application.test-samples';

import { JobApplicationFormService } from './job-application-form.service';

describe('JobApplication Form Service', () => {
  let service: JobApplicationFormService;

  beforeEach(() => {
    service = TestBed.inject(JobApplicationFormService);
  });

  describe('Service methods', () => {
    describe('createJobApplicationFormGroup', () => {
      it('should create a new form with FormControl', () => {
        const formGroup = service.createJobApplicationFormGroup();

        expect(formGroup.controls).toEqual(
          expect.objectContaining({
            id: expect.any(Object),
            userId: expect.any(Object),
            status: expect.any(Object),
            coverLetter: expect.any(Object),
            notes: expect.any(Object),
            matchScore: expect.any(Object),
            createdAt: expect.any(Object),
            updatedAt: expect.any(Object),
            appliedAt: expect.any(Object),
            jobOffer: expect.any(Object),
            candidateProfile: expect.any(Object),
          }),
        );
      });

      it('passing IJobApplication should create a new form with FormGroup', () => {
        const formGroup = service.createJobApplicationFormGroup(sampleWithRequiredData);

        expect(formGroup.controls).toEqual(
          expect.objectContaining({
            id: expect.any(Object),
            userId: expect.any(Object),
            status: expect.any(Object),
            coverLetter: expect.any(Object),
            notes: expect.any(Object),
            matchScore: expect.any(Object),
            createdAt: expect.any(Object),
            updatedAt: expect.any(Object),
            appliedAt: expect.any(Object),
            jobOffer: expect.any(Object),
            candidateProfile: expect.any(Object),
          }),
        );
      });
    });

    describe('getJobApplication', () => {
      it('should return NewJobApplication for default JobApplication initial value', () => {
        const formGroup = service.createJobApplicationFormGroup(sampleWithNewData);

        const jobApplication = service.getJobApplication(formGroup);

        expect(jobApplication).toMatchObject(sampleWithNewData);
      });

      it('should return NewJobApplication for empty JobApplication initial value', () => {
        const formGroup = service.createJobApplicationFormGroup();

        const jobApplication = service.getJobApplication(formGroup);

        expect(jobApplication).toMatchObject({});
      });

      it('should return IJobApplication', () => {
        const formGroup = service.createJobApplicationFormGroup(sampleWithRequiredData);

        const jobApplication = service.getJobApplication(formGroup);

        expect(jobApplication).toMatchObject(sampleWithRequiredData);
      });
    });

    describe('resetForm', () => {
      it('passing IJobApplication should not enable id FormControl', () => {
        const formGroup = service.createJobApplicationFormGroup();
        expect(formGroup.controls.id.disabled).toBe(true);

        service.resetForm(formGroup, sampleWithRequiredData);

        expect(formGroup.controls.id.disabled).toBe(true);
      });

      it('passing NewJobApplication should disable id FormControl', () => {
        const formGroup = service.createJobApplicationFormGroup(sampleWithRequiredData);
        expect(formGroup.controls.id.disabled).toBe(true);

        service.resetForm(formGroup, { id: null });

        expect(formGroup.controls.id.disabled).toBe(true);
      });
    });
  });
});
